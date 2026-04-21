# BitCraft 服务端完整技术分析文档

> **项目仓库：** [clockworklabs/BitCraftPublic](https://github.com/SABERBOY/BitCraftPublic)  
> **技术栈：** Rust · SpacetimeDB 1.12.0 · WebAssembly  

---

## 目录

1. [项目概述与技术栈](#1-项目概述与技术栈)
2. [双模块架构设计](#2-双模块架构设计)
3. [SpacetimeDB 核心概念速查](#3-spacetimedb-核心概念速查)
4. [实体组件系统（ECS）](#4-实体组件系统ecs)
5. [完整数据模型（Tables）](#5-完整数据模型tables)
6. [坐标系与空间系统](#6-坐标系与空间系统)
7. [Reducer Handler 系统](#7-reducer-handler-系统)
8. [定时 Agent 系统](#8-定时-agent-系统)
9. [跨模块通信系统](#9-跨模块通信系统)
10. [世界生成系统](#10-世界生成系统)
11. [自定义过程宏（bitcraft-macro）](#11-自定义过程宏bitcraft-macro)
12. [构建系统（Build System）](#12-构建系统build-system)
13. [静态数据导入系统](#13-静态数据导入系统)
14. [权限与安全模型](#14-权限与安全模型)
15. [Global 模块完整分析](#15-global-模块完整分析)
16. [构建、发布、客户端绑定命令速查](#16-构建发布客户端绑定命令速查)

---

## 1. 项目概述与技术栈

BitCraft 是一款社区沙盒 MMORPG，其后端完全基于 [SpacetimeDB](https://spacetimedb.com/docs/) 构建。所有游戏状态存储在 SpacetimeDB **Tables** 中，所有变更通过 **Reducers**（Rust 函数）执行。没有传统 HTTP API 或额外应用层服务器——SpacetimeDB 本身就是应用服务器。

### 技术选型

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | stable | 主要编程语言 |
| SpacetimeDB | `=1.12.0`（固定） | 响应式实时后端数据库 |
| WebAssembly (cdylib) | - | 部署格式，模块编译为 WASM |
| bitcraft-macro | 内部 crate | 自定义过程宏 |
| strum/strum_macros | 0.24 | 枚举迭代与派生 |
| OpenSimplex / Octave Noise | 自实现 | 世界生成噪声算法 |

### 项目规模

| 模块 | Rust 文件数 | 说明 |
|------|------------|------|
| `packages/game/` | 551 | 区域游戏模块（每区域一实例） |
| `packages/global_module/` | 151 | 全局协调模块（集群唯一） |
| Handler 文件 | 278 | 跨 21 个业务域的 Reducer |

---

## 2. 双模块架构设计

BitCraft 由两个独立的 SpacetimeDB 模块构成，通过异步消息互联：

```
┌─────────────────────────────────────┐           ┌──────────────────────────────┐
│    global_module（每集群一个）        │◄─────────►│   game（每游戏区域一个）       │
│                                     │  inter-   │                              │
│  ● 帝国系统（EmpireState）           │  module   │  ● 玩家移动、战斗、制作        │
│  ● 攻城逻辑（Siege）                 │  messages │  ● 建筑、领地                 │
│  ● 跨区玩家追踪（UserRegionState）   │           │  ● 物品、技能、装备            │
│  ● 人口统计（RegionPopulationInfo）  │           │  ● 世界生成（初始化时）         │
│  ● 玩家命名与审核                    │           │  ● NPC/怪物 AI                │
│  ● 好友/封禁系统                    │           │  ● 资源采集                   │
│  ● 聊天频道管理                     │           │                              │
└─────────────────────────────────────┘           └──────────────────────────────┘
```

### 为何如此拆分？

- **game 模块**处理高频、实时的玩家交互（每秒多次 reducer 调用），需要低延迟。
- **global_module** 处理低频的全局协调事务（帝国创建、攻城周期、玩家注册），一个集群只需一个实例。
- 通过异步消息（而非同步 RPC）解耦，防止一个模块的延迟影响另一个模块。

### 通信方式

跨模块通信是**纯异步消息传递**，通过 `inter_module_message_v2` 表：

```
发送方 ─ insert InterModuleMessageV2 ──► SpacetimeDB 路由
接收方 ◄── 触发 process_inter_module_message reducer ──
```

消息**永不同步等待**，失败的消息存储在 `inter_module_message_errors` 中，不会回滚发送方事务。

---

## 3. SpacetimeDB 核心概念速查

### Table（表）

用 `#[spacetimedb::table]` 标注的 Rust struct，存储在 SpacetimeDB 数据库中：

```rust
#[spacetimedb::table(name = player_state, public)]
pub struct PlayerState {
    #[primary_key]
    pub entity_id: u64,
    pub signed_in: bool,
    pub time_played: i32,
}
```

- `public` — 客户端可以订阅此表
- `#[primary_key]` — 唯一主键
- `#[auto_inc]` — 自增主键
- `#[index(btree)]` / `#[index(hash)]` — 添加索引

### Reducer（变更函数）

用 `#[spacetimedb::reducer]` 标注的 Rust 函数，是所有状态变更的**唯一入口**：

```rust
#[spacetimedb::reducer]
pub fn player_move(ctx: &ReducerContext, request: PlayerMoveRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;
    // 读写 ctx.db.xxx() 表
    Ok(())
}
```

- Reducer 失败（返回 `Err`）→ 整个事务**回滚**，不产生任何副作用。
- `ReducerContext` 提供：数据库访问（`ctx.db`）、时间戳（`ctx.timestamp`）、发送方身份（`ctx.sender`）。

### Scheduled Reducer（定时 Reducer）

通过特殊 timer 表实现的服务端定时循环：

```rust
#[spacetimedb::table(name = my_timer, scheduled(my_agent_tick))]
pub struct MyTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

#[spacetimedb::reducer]
pub fn my_agent_tick(ctx: &ReducerContext, _timer: MyTimer) -> Result<(), String> {
    // 定期执行的逻辑
    // 在末尾重新插入 timer 以实现持续循环
    Ok(())
}
```

### cdylib / WASM 部署

两个模块都编译为 `cdylib`（WebAssembly 动态库），通过 `spacetime build` 和 `spacetime publish` 部署。SpacetimeDB 版本固定为 `=1.12.0`（在两个 `Cargo.toml` 中均固定）。

---

## 4. 实体组件系统（ECS）

BitCraft 不使用传统游戏对象，而是用纯 ECS 模式：每个实体是一个 `u64` 实体 ID，其属性分散在多个独立表中作为"组件"。

### 实体 ID 编码

```rust
pub fn create_entity(ctx: &ReducerContext) -> u64 {
    let mut globals = ctx.db.globals().version().find(&0).unwrap();
    globals.entity_pk_counter += 1;
    let pk = globals.entity_pk_counter;
    let pk = pk | ((globals.region_index as u64) << 56);  // 高8位 = 区域索引
    ctx.db.globals().version().update(globals);
    pk
}
```

| 位段 | 含义 |
|------|------|
| 高 8 位（bit 56–63） | region_index（所属区域，最多 255 个区域） |
| 低 56 位（bit 0–55） | 区域内单调递增计数器 |

**恢复区域：** `region_id = (entity_id >> 56) as u8`

### 典型实体的组件构成

一个玩家实体（entity_id = X）会同时出现在以下表中：

```
UserState               - 身份认证 & 登录控制
PlayerState             - 会话时间、传送点
PlayerUsernameState     - 用户名
HealthState             - 生命值
StaminaState            - 体力
MobileEntityState       - 当前位置与移动目标（动态实体）
InventoryState          - 物品栏（多个，inventory_index 区分）
EquipmentState          - 装备槽
ExperienceState         - 技能经验值（多技能）
CharacterStatsState     - 属性数值（伤害、护甲等）
SignedInPlayerState     - 在线标志（存在即在线）
KnowledgeBuildingState  - 建筑发现记录
KnowledgeCraftState     - 配方发现记录
... 等 15+ 个知识类组件
```

### actor_id 解析

所有玩家 reducer 的标准开头：

```rust
pub fn actor_id(ctx: &ReducerContext, must_be_signed_in: bool) -> Result<u64, String> {
    match ctx.db.user_state().identity().find(&ctx.sender) {
        Some(user) => {
            if must_be_signed_in {
                ensure_signed_in(ctx, user.entity_id)?;  // 检查 SignedInPlayerState
            }
            Ok(user.entity_id)
        }
        None => Err("Invalid sender".into()),
    }
}
```

调用方式：
```rust
let actor_id = game_state::actor_id(&ctx, true)?;  // 必须在线
let actor_id = game_state::actor_id(&ctx, false)?; // 允许未登录（账号创建流程）
```

---

## 5. 完整数据模型（Tables）

### 5.1 玩家状态表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `UserState` | `entity_id` | `identity(unique)`, `can_sign_in` | 认证状态，共享表（区域→全局复制） |
| `PlayerState` | `entity_id` | `teleport_location`, `time_played`, `signed_in` | 玩家核心状态 |
| `PlayerUsernameState` | `entity_id` | `username(unique)` | 用户名映射 |
| `PlayerLowercaseUsernameState` | `entity_id` | `username_lowercase(unique)` | 大小写不敏感查找 |
| `HealthState` | `entity_id` | `health: f32`, `died_timestamp: i32` | 生命值（玩家/敌人/NPC通用） |
| `StaminaState` | `entity_id` | `stamina: f32`, `last_decrease_timestamp` | 体力值 |
| `TeleportationEnergyState` | `entity_id` | `energy: f32` | 传送能量 |
| `SatiationState` | `entity_id` | `satiation: f32` | 饥饿/饱食度 |
| `ExperienceState` | `entity_id` | `experience_stacks: Vec<ExperienceStack>` | 多技能经验 |
| `PartialExperienceState` | `entity_id` | `experience_stacks: Vec<ExperienceStackF32>` | 小数经验积累（精度处理） |
| `CharacterStatsState` | `entity_id` | `values: Vec<f32>` | 属性数值向量 |
| `SignedInPlayerState` | `entity_id` | — | 在线标志 |
| `PlayerPrefsState` | `entity_id` | `default_deployable_collectible_id` | 默认可部署收藏品 |
| `PlayerSettingsState` | `entity_id` | `fill_player_inventory`, `fill_deployable_inventory_first` | 游戏设置 |
| `RezSickLongTermState` | `entity_id` | — | 复活病（长期减益优化标志） |
| `StarvingPlayerState` | `entity_id` | — | 饥饿状态标志（优化用） |

### 5.2 位置/移动表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `LocationState` | `entity_id` | `chunk_index`, `x`, `z`, `dimension` | 静态实体位置。索引：`(x,z,chunk_index)`, `(chunk_index)` |
| `MobileEntityState` | `entity_id` | `location_x/z`, `destination_x/z`, `dimension`, `is_walking`, `chunk_index` | 移动实体（玩家/敌人/NPC）。repr(C) 48字节打包 |
| `MoveValidationStrikeCounterState` | `entity_id` | `validation_failure_timestamps: Vec<Timestamp>` | 反作弊移动验证 |

> ⚠️ **重要设计决策：** `LocationState.dimension` **故意不建立索引**。按 dimension 建索引会导致大规模时严重性能退化。务必使用 `dimension_filter()` 辅助函数替代原始维度查询。

### 5.3 建筑状态表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `BuildingState` | `entity_id` | `claim_entity_id`, `building_description_id`, `direction_index` | 建筑实例（共享表） |
| `BuildingNicknameState` | `entity_id` | `nickname: String` | 玩家自定义建筑名（共享表） |
| `ResourceHealthState` | `entity_id` | `health: i32` | 建筑/资源耐久（区别于 HealthState） |
| `LootChestState` | `entity_id` | `building_entity_id`, `loot_chest_id`, `building_spawn_id` | 战利品箱状态 |
| `PortalState` | `entity_id` | `target_building_entity_id`, `destination_x/z/dimension`, `enabled` | 传送门链接 |
| `WaystoneState` | `entity_id` | `building_entity_id` | 路标石（传送点）状态 |
| `SignState` | `entity_id` | `text: String` | 告示牌文本 |
| `BankState` | `entity_id` | `building_entity_id` | 银行关联状态 |
| `MarketplaceState` | `entity_id` | `building_entity_id` | 市场关联状态 |

### 5.4 战斗/敌人表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `EnemyState` | `entity_id` | `herd_entity_id`, `enemy_type`, `last_ranged_attack_timestamp` | 敌人实例 |
| `EnemyScalingState` | `entity_id` | `enemy_scaling_id` | 敌人难度缩放 |
| `HerdState` | `entity_id` | `enemy_ai_params_desc_id` | 敌人群体（多个敌人共享AI参数） |
| `AttachedHerdsState` | `entity_id` | `herds_entity_ids: Vec<u64>` | 多群体附加到一个实体 |
| `TargetState` | `entity_id` | `target_entity_id` | 战斗目标（攻击对象） |
| `ThreatState` | `entity_id` | `owner_entity_id`, `target_entity_id`, `threat: f32` | 仇恨值追踪（AI决策用） |
| `CombatState` | `entity_id` | `last_attacked_timestamp`, `global_cooldown` | 战斗计时状态 |
| `CombatImmunityState` | `entity_id` | `immunity_end_timestamp`, `crumb_trail_entity_id` | 无敌/豁免窗口 |
| `AttackOutcomeState` | `entity_id` | `damage`, `crit_result`, `dodge_result` | 最近一次攻击结果 |
| `ExtractOutcomeStateV2` | `entity_id` | `target_entity_id`, `damage`, `is_crit` | 采集/挖掘结果 |
| `ContributionState` | `entity_id` | `player_entity_id`, `enemy_entity_id`, `contribution: f32` | 战利品贡献追踪 |
| `DuelState` | `entity_id` | `initiator_entity_id`, `acceptor_entity_id`, `victor` | PVP决斗状态 |
| `CombatDimensionState` | `dimension_id` | — | 允许战斗的维度标志 |

### 5.5 物品/背包表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `InventoryState` | `entity_id` | `pockets: Vec<Pocket>`, `inventory_index`, `cargo_index`, `owner_entity_id`, `player_owner_entity_id` | 物品容器（玩家、箱子等） |
| `EquipmentState` | `entity_id` | `equipment_slots: Vec<EquipmentSlot>` | 装备槽 |
| `EquipmentPresetState` | `entity_id` | `player_entity_id`, `index`, `active` | 装备预设方案 |
| `DroppedInventoryState` | `entity_id` | `owner_entity_id`, `active_timer_id` | 掉落在地面的物品 |
| `LostItemsState` | `inventory_entity_id` | `owner_entity_id`, `location` | 死亡掉落物追踪 |
| `TradeSessionState` | `entity_id` | `initiator_entity_id`, `acceptor_entity_id`, `status`, `initiator_offer`, `acceptor_offer` | 玩家间交易会话 |

### 5.6 领地/帝国表（共享表）

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `ClaimState` | `entity_id` | `owner_building_entity_id(unique)`, `name(unique)`, `owner_player_entity_id`, `neutral` | 领地实例（共享表，区域所有） |
| `ClaimLowercaseNameState` | `entity_id` | `name_lowercase(unique)` | 领地名大小写不敏感（全局所有） |
| `ClaimTileState` | `entity_id` | `claim_id` | 每个地块的领地归属 |
| `ClaimLocalState` | `entity_id` | `supplies`, `treasury`, `num_tiles`, `building_maintenance` | 领地本地数据（供给、国库） |
| `ClaimMemberState` | `entity_id` | `claim_entity_id`, `player_entity_id`, 各种权限 bool | 领地成员及权限 |
| `ClaimTechState` | `entity_id` | `learned: Vec<i32>`, `researching`, `start_timestamp` | 领地科技树进度 |
| `EmpireState` | `entity_id` | `capital_building_entity_id(unique)`, `name(unique)`, `shard_treasury` | 帝国核心（全局所有，共享表） |
| `EmpirePlayerDataState` | `entity_id` | `empire_entity_id`, `rank`, `donated_shards`, `noble` | 玩家帝国成员数据 |
| `EmpireRankState` | `entity_id` | `empire_entity_id`, `rank`, `title`, `permissions: Vec<bool>` | 帝国等级定义 |
| `EmpireNodeState` | `entity_id` | `empire_entity_id`, `chunk_index`, `energy`, `active`, `upkeep` | 帝国瞭望塔节点（控制领土） |
| `EmpireChunkState` | `chunk_index` | `empire_entity_id`, `watchtower_entity_id` | 地块级帝国控制 |
| `EmpireSettlementState` | `building_entity_id` | `claim_entity_id(unique)`, `empire_entity_id` | 帝国定居点（帝国内的领地） |
| `EmpireNodeSiegeState` | `entity_id` | `building_entity_id`, `empire_entity_id`, `energy`, `active` | 攻城状态（共享表） |

### 5.7 可部署物（Deployable）表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `DeployableState` | `entity_id` | `owner_id`, `claim_entity_id`, `deployable_description_id`, `nickname`, `hidden` | 部署到世界的对象（车、坐骑、摊位等） |
| `DeployableCollectibleState` | `deployable_entity_id` | `owner_entity_id`, `collectible_id`, `auto_follow` | 可收藏宠物/伙伴 |
| `MountingState` | `entity_id` | `deployable_entity_id`, `deployable_slot` | 骑乘/乘坐状态 |

### 5.8 行动/制作表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `PlayerActionState` | `auto_id(auto_inc)` | `entity_id`, `start_time`, `duration`, `action_type`, `layer` | 玩家进行中的行动（制作、采集等）。repr(C) 72字节打包 |
| `ProgressiveActionState` | `entity_id` | `building_entity_id`, `progress`, `recipe_id`, `craft_count`, `lock_expiration` | 建筑制作进度（多步骤行动） |
| `PassiveCraftState` | `entity_id` | `owner_entity_id`, `recipe_id`, `building_entity_id`, `status` | 后台被动制作队列 |
| `ProjectSiteState` | `entity_id` | `construction_recipe_id`, `items`, `progress`, `owner_id` | 建筑工地进度 |
| `TerraformProgressState` | `entity_id` | `final_height_target`, `progress` | 地形改造进度 |
| `ActionState` | `entity_id` | `owner_entity_id`, `action_id`, `cooldown` | 战斗技能冷却 |
| `AbilityState` | `entity_id` | `owner_entity_id`, `ability`, `cooldown` | 能力/技能冷却 |
| `ActionBarState` | `entity_id` | `player_entity_id`, `action_bar_index`, `ability_entity_id` | 快捷栏配置 |

### 5.9 NPC/商人表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `NpcState` | `entity_id` | `npc_type`, `building_entity_id`, `traveling`, `previous_buildings` | NPC 状态（商人、任务给予者） |
| `TradeOrderState` | `entity_id` | `shop_entity_id`, `offer_items`, `required_items`, `remaining_stock` | 商店交易单 |
| `BarterStallState` | `entity_id` | `market_mode_enabled` | 玩家摊位状态 |
| `SellOrderState` | `entity_id` | `owner_entity_id`, `claim_entity_id`, `item_id`, `price_threshold`, `quantity` | 市场卖单 |
| `BuyOrderState` | `entity_id` | 同上 | 市场买单 |
| `ClosedListingState` | `entity_id` | `owner_entity_id`, `item_stack`, `timestamp` | 历史成交记录 |

### 5.10 知识/发现表

BitCraft 有 15 个以上的 `Knowledge*State` 表，每种记录不同类型的发现（建筑、制作、资源、敌人、地点、成就等）：

```
KnowledgeBattleActionState   KnowledgeBuildingState   KnowledgeCraftState
KnowledgeCargoState          KnowledgeEnemyState       KnowledgeExtractState
KnowledgeItemState           KnowledgeLoreState        KnowledgeNpcState
KnowledgeResourceState       KnowledgeRuinsState       KnowledgeClaimState
KnowledgeSecondaryState      KnowledgeConstructionState  KnowledgeAchievementState
```

每个表结构相同：
```rust
pub struct KnowledgeXxxState {
    #[primary_key]
    pub entity_id: u64,
    pub entries: Vec<KnowledgeEntry>,
}

pub struct KnowledgeEntry {
    pub id: i32,                  // 知识 ID（对应静态数据表）
    pub state: KnowledgeState,    // Discovered / Acquired
}
```

### 5.11 地形/世界表

| 表名 | 主键 | 关键字段 | 说明 |
|------|------|---------|------|
| `TerrainChunkState` | `chunk_index: u64` | 地形数据数组（每块 32×32 小地块） | 世界地形块 |
| `PavingTileState` | `entity_id` | `coordinates`, `claim_entity_id`, `paving_type` | 玩家铺设的地面 |
| `PillarState` | `entity_id` | `coordinates`, `claim_entity_id`, `pillar_type` | 装饰柱 |
| `ResourceState` | `entity_id` | `resource_description_id`, `cluster_id` | 可采集资源节点 |
| `DimensionDescriptionState` | `dimension_id` | `dimension_size_large_x/z` | 内部空间（地下城等）维度描述 |

---

## 6. 坐标系与空间系统

### 四级六边形网格

```
FloatHexTile          —— 浮点精度（寻路用）
     ↓ 量化
SmallHexTile (x, z)   —— 单个地形格（精细碰撞）
     ↓ ×32
LargeHexTile          —— 大地形格（地形块内坐标）
     ↓ ×32
ChunkCoordinates      —— 世界区块 (32×32 LargeHexTile)
```

所有坐标都附带 `dimension: u32` 字段，用于区分覆盖世界（0）与内部空间（>0）。

### HexCoordinates 操作

```rust
pub struct HexCoordinates {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

// 主要操作
hex.neighbor(direction: HexDirection)     // 获取6个相邻格之一
hex.distance_to(other: HexCoordinates)   // 六边形距离
hex.to_offset_coordinates()               // 转换为标准偏移坐标
hex.from_offset_coordinates(x, z, scale) // 从偏移坐标转换
```

### 空间缓存（LocationStateCache）

为避免全表扫描，所有空间查询使用缓存：

```rust
// 错误做法（性能差）：
ctx.db.location_state().iter().filter(|l| l.chunk_index == target_chunk)

// 正确做法：
LocationStateCache::get_entities_in_chunk(ctx, chunk_index)
ClaimTileStateCache::get_claim_for_tile(ctx, x, z)
```

### 坐标辅助函数（game_state_filters.rs）

```rust
coordinates(ctx, entity_id) -> SmallHexTile           // 静态实体位置
coordinates_float(ctx, entity_id) -> FloatHexTile      // 浮点位置（移动实体）
coordinates_any(ctx, entity_id) -> SmallHexTile        // 任意类型实体
building_at_coordinates(ctx, coords) -> Option<BuildingState>
project_site_at_coordinates(ctx, coords) -> Option<ProjectSiteState>
dimension_filter(iter, dimension) -> impl Iterator     // 维度过滤（代替索引）
```

---

## 7. Reducer Handler 系统

### 标准 Reducer 模式

```rust
#[spacetimedb::reducer]
#[feature_gate("build")]           // 可选：功能门控
pub fn building_deconstruct(
    ctx: &ReducerContext,
    request: PlayerBuildingDeconstructRequest
) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;      // 1. 解析玩家
    let building = ctx.db.building_state()                  // 2. 查询目标
        .entity_id().find(&request.building_entity_id)
        .ok_or("Building not found")?;
    // 3. 权限检查
    permission_helper::check_build_permission(ctx, actor_id, building.claim_entity_id)?;
    // 4. 业务逻辑（修改状态）
    ctx.db.building_state().entity_id().delete(building.entity_id);
    inventory_helpers::add_items(ctx, actor_id, return_materials)?;
    Ok(())
}
```

### 21 个 Handler 域完整列表

#### Domain: admin（46 个 Reducer）

管理员专用操作，全部需要 `has_role(Admin)` 权限：

| Reducer | 说明 |
|---------|------|
| `admin_broadcast_msg_region` | 向区域玩家广播消息 |
| `admin_cap_skill` | 批量限制玩家技能等级 |
| `admin_clear_all_resources` | 清除所有资源（分块定时执行） |
| `admin_collapse_ruins` | 折叠所有废墟内部空间 |
| `admin_count_inventory_items` | 统计所有玩家持有特定物品数量 |
| `admin_create_player_report` | 创建玩家举报（发送至全局模块） |
| `admin_delete_all_items_of_type` | 删除玩家所有特定类型物品 |
| `admin_despawn_overworld_enemies` | 清除所有地面敌人 |
| `admin_find_items_in_inventories` | 审计：搜索物品在背包中的分布 |
| `admin_grant_collectibles` | 授予玩家收藏品 |
| `admin_manage_developers` | 授予/撤销开发者角色 |
| `admin_quests` | 操作玩家任务进度 |
| `admin_rename_building/claim/deployable` | 重命名游戏对象 |
| `admin_resource_force_regen` | 强制刷新指定区块资源 |
| `admin_restore_all_buildings_health` | 修复所有建筑至满血 |
| `admin_set_region_control_state` | 控制区域访问状态 |
| `admin_sign_out` / `admin_sign_out_all` | 强制玩家下线 |
| `update_chunk_index` | 重新计算所有实体的区块索引 |
| ... 等 28+ 个 | |

#### Domain: buildings（10 个 Reducer）

| Reducer | 宏 | 说明 |
|---------|----|----|
| `building_deconstruct_start` | `#[feature_gate("build")]` | 开始拆除建筑 |
| `building_deconstruct` | `#[shared_table_reducer]`, `#[feature_gate]` | 完成拆除，返还材料 |
| `building_move` | `#[shared_table_reducer]`, `#[feature_gate]` | 移动建筑位置 |
| `building_repair_start` | `#[feature_gate]` | 开始修复建筑 |
| `building_repair` | `#[feature_gate]` | 执行修复（恢复耐久） |
| `building_set_nickname` | — | 重命名建筑 |
| `building_set_sign_text` | — | 修改告示牌文字 |
| `project_site_place` | `#[feature_gate]` | 放置施工地基 |
| `project_site_add_materials` | — | 向工地投入材料 |
| `project_site_advance_project` | — | 完成施工，生成建筑实体 |

#### Domain: claim（22 个 Reducer）

| Reducer | 说明 |
|---------|------|
| `claim_add_member` | 添加玩家为领地成员 |
| `claim_add_tile` | 扩展领地范围（消耗材料） |
| `claim_remove_member` | 踢出成员 |
| `claim_remove_tile` | 缩减领地范围 |
| `claim_rename` | 重命名领地 |
| `claim_resupply` | 向领地补充供给 |
| `claim_set_member_permissions` | 修改成员权限（建造/物品/军官/共同所有者） |
| `claim_set_protection_threshold` | 设置领地保护阈值 |
| `claim_take_ownership` | 接管领地所有权 |
| `claim_tech_learn` | 开始研究领地科技 |
| `claim_tech_cancel` | 取消科技研究 |
| `claim_tech_unlock_tech` | 解锁已完成的科技 |
| `claim_transfer_ownership` | 将领地转让给其他玩家 |
| `claim_treasury_deposit` | 向国库存入金币 |
| `claim_withdraw_from_treasury` | 从国库提取金币 |
| `claim_purchase_supplies_from_player` | 从其他玩家购买供给 |
| ... 等 | |

#### Domain: empires（10 个 Reducer）

| Reducer | 跨模块 | 说明 |
|---------|--------|------|
| `empire_create` | ✅ → Global | 从领地创建帝国 |
| `empire_claim_join` | ✅ → Global | 加入帝国 |
| `empire_collect_hexite_capsule` | — | 采集六边形能源胶囊 |
| `empire_donate_item` | — | 捐献物品到帝国国库 |
| `empire_queue_supplies` | — | 将供给加入帝国分发队列 |
| `empire_resupply_node` | — | 从节点收取补给 |
| `empire_siege_add_supplies` | — | 向攻城补充供给 |
| `empire_start_siege` | ✅ → Global | 发起对瞭望塔的攻城 |
| `empire_withdraw_item` | — | 从帝国国库提取物品 |

#### Domain: player（59+ 个 Reducer）

| Reducer | 宏 | 说明 |
|---------|----|----|
| `sign_in` | `#[shared_table_reducer]`, `#[feature_gate]` | 玩家登录初始化 |
| `sign_out` | — | 结束会话 |
| `player_move` | `#[feature_gate]` | 更新玩家位置（移动验证+体力检查） |
| `player_death_start` | `#[table(scheduled)]` | 死亡序列（定时器触发） |
| `player_respawn` | — | 复活 |
| `player_climb` | — | 爬越地形 |
| `eat` | — | 消耗食物（恢复体力/生命） |
| `extract` | — | 从资源节点采集材料 |
| `emote` | — | 播放表情动画 |
| `chat_post_message` | — | 发送聊天消息（审核过滤） |
| `order_post_buy_order` / `order_post_sell_order` | — | 市场挂单 |
| `order_cancel` / `order_collect` | — | 撤单/收取 |
| `closed_listing_collect` | — | 领取市场成交物品 |
| `player_duel_initiate` | — | 发起 PVP 决斗 |
| `player_teleport_home` | — | 传送回家 |
| `player_teleport_waystone` | — | 传送至路标石 |
| `player_use_elevator` | `#[table(scheduled)]` | 使用电梯 |
| `player_region_crossover` | ✅ 跨区 | 跨区域移动 |
| `terraform_start` | `#[feature_gate("terraform")]` | 开始地形改造 |
| `paving_place_tile` | — | 铺设地面 |
| `portal_enter` | — | 进入传送门（切换维度） |
| `set_home` | — | 设置重生点 |
| `sleep` | — | 休息恢复体力 |
| `achievement_claim` | — | 领取成就奖励 |
| `report_player` | ✅ → Global | 举报玩家 |
| `knowledge_acquire_from_entities` | — | 从实体学习知识 |
| `ability_set` / `ability_remove` | — | 技能热键管理 |
| `player_housing_enter` | — | 进入玩家住宅 |
| `player_vote_answer` | — | 参与区域投票 |
| `scroll_read` | — | 阅读技能书卷 |
| `player_action_cancel` | — | 取消当前行动 |
| ... 等 | |

#### Domain: player_craft（11 个 Reducer）

| Reducer | 说明 |
|---------|------|
| `craft` | 开始手工制作（消耗时间，触发 PlayerActionState） |
| `craft_complete` | 完成制作，产出物品 |
| `craft_cancel` | 取消制作（退还部分材料） |
| `craft_collect` | 领取已完成的制作品 |
| `passive_craft_queue` | 将配方加入被动制作队列 |
| `passive_craft_cancel` | 取消被动制作 |
| `passive_craft_collect` | 领取被动制作完成品 |
| `passive_craft_collect_all` | 批量领取所有完成品 |
| `passive_craft_process` | 定时器：推进被动制作进度 |
| `item_convert` | 物品转换（外观变换） |

#### Domain: player_trade（13 个 Reducer）

| Reducer | 说明 |
|---------|------|
| `trade_initiate_session` | 发起与另一玩家的交易 |
| `trade_add_item` / `trade_remove_item` | 修改交易报价 |
| `trade_accept` | 接受并完成交易 |
| `trade_decline` | 拒绝交易 |
| `barter_stall_order_create` | 在摊位上创建以物易物单 |
| `barter_stall_order_accept` | 接受摊位交易 |
| `barter_stall_order_delete` | 撤销摊位交易单 |

#### Domain: player_vault（12 个 Reducer）

| Reducer | 说明 |
|---------|------|
| `collectible_activate` | 召唤收藏品（宠物/伙伴） |
| `deployable_deploy` | 将收藏品部署到世界 |
| `deployable_store` | 将部署物收回收藏品库 |
| `deployable_hide` | 临时隐藏部署物 |
| `deployable_toggle_auto_follow` | 切换宠物跟随模式 |
| `convert_collectible_to_deed` | 收藏品转为可交易票据 |
| `convert_deed_to_collectible` | 票据转为收藏品 |

#### Domain: attack（3 个 Reducer）

| Reducer | 宏 | 说明 |
|---------|----|----|
| `attack_single` | `#[feature_gate("combat")]` | 单体攻击 |
| `attack_area` | `#[feature_gate("combat")]` | AOE 攻击 |
| `attack_structure` | `#[feature_gate("combat")]` | 攻击建筑/结构 |

#### Domain: server（15 个 Reducer，服务端内部调用）

| Reducer | 说明 |
|---------|------|
| `enemy_spawn` / `enemy_spawn_batch` | 生成敌人实体 |
| `enemy_despawn` | 清除敌人 |
| `enemy_move` | 更新敌人位置（AI 驱动） |
| `enemy_set_health` | 设置敌人生命值 |
| `loot_chest_spawn` / `loot_chest_despawn` | 生成/清除战利品箱 |
| `building_despawn` | 清除建筑实体 |
| `on_durability_zero` | 工具耐久归零处理 |
| `server_teleport_player` | 强制传送玩家 |
| `interior_set_collapsed` | 设置内部空间折叠状态 |
| `destroy_dimension_network` | 删除维度网络 |

#### 其他域（概要）

| 域 | 说明 |
|----|------|
| `cheats` (43) | 开发/测试工具，仅限管理员/开发环境 |
| `inventory` (10) | 物品操作（丢弃、拾取、移动、分割） |
| `migration` (8) | 数据迁移脚本（版本升级时执行） |
| `dev` (2) | 蓝图放置、世界清除（开发工具） |
| `queue` (4) | 登录队列管理（加入、离开、宽限期超时） |
| `rentals` (1) | 租赁系统（定时扣款） |
| `resource` (1) | 区块资源刷新 |
| `stats` (1) | 领地统计数据重算 |
| `world` (3) | 世界生成专用（放置建筑/资源/NPC瞭望塔） |

---

## 8. 定时 Agent 系统

### 架构模式

每个 Agent 有一个对应的 timer 表和 reducer：

```rust
// 定义 timer 表
#[spacetimedb::table(name = my_agent_timer, scheduled(my_agent_tick))]
pub struct MyAgentTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// 定义 reducer（每次 timer 触发时调用）
#[spacetimedb::reducer]
pub fn my_agent_tick(ctx: &ReducerContext, _timer: MyAgentTimer) -> Result<(), String> {
    if !agents::should_run(ctx) { return Ok(()); }  // 检查全局开关
    // 业务逻辑...
    // 重新插入 timer 实现持续循环
    ctx.db.my_agent_timer().insert(MyAgentTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(tick_ms)),
    });
    Ok(())
}
```

所有 Agent 通过 `agents::should_run(ctx)` 检查 `config.agents_enabled` 全局开关。

### game 模块 Agent 列表

| Agent | tick 频率 | 关键功能 |
|-------|----------|---------|
| `auto_logout_agent` | 30 秒 | 踢出超时空闲玩家 |
| `building_decay_agent` | 可配置 | 扣除无领地建筑的耐久 |
| `player_regen_agent` | 1 秒 | 玩家生命/体力自然回复 |
| `enemy_regen_agent` | 1 秒 | 敌人生命回复 |
| `npc_ai_agent` | 5 分钟 | NPC 在废墟间移动 |
| `resources_regen` | 可配置 | 刷新耗尽的资源节点 |
| `day_night_agent` | 自适应 | 白天/黑夜循环 |
| `region_population_agent` | 5 秒 | 向全局模块广播玩家人数 |
| `claim_decay_agent` | 可配置 | 消耗领地供给 |
| `building_loot_agent` | 可配置 | 生成/刷新建筑内战利品 |
| `combat_agent` | ~100ms | NPC 战斗逻辑 |
| `herd_movement_agent` | 可配置 | 怪物群体移动 |
| `passive_craft_agent` | 可配置 | 推进被动制作进度 |
| `marketplace_agent` | 可配置 | 处理市场到期订单 |
| `claim_tech_agent` | 可配置 | 完成领地科技研究 |
| `dropped_item_cleanup_agent` | 可配置 | 清除过期掉落物 |
| `traveler_agent` | 可配置 | 旅行商人逻辑 |
| `buff_agent` | 可配置 | 处理 buff 到期 |

所有 tick 频率均从 `parameters_desc` 静态数据表读取，管理员可通过 `update_scheduled_timers_from_static_data` reducer 热更新。

### global_module Agent 列表

| Agent | 功能 |
|-------|------|
| `empire_decay_agent` | 定期扣除瞭望塔节点能量（upkeep），能量归零后节点停用 |
| `empire_siege_agent` | 处理攻城进度，扣除攻防双方供给，攻城成功/失败判定 |

---

## 9. 跨模块通信系统

### InterModuleMessageV2 协议

```rust
#[spacetimedb::table(name = inter_module_message_v2)]
pub struct InterModuleMessageV2 {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub to: u8,                    // 目标模块 ID（0 = global）
    pub contents: MessageContentsV2,
}

pub enum InterModuleDestination {
    Global,                        // 仅发往 global_module
    AllOtherRegions,               // 广播至所有其他区域
    GlobalAndAllOtherRegions,      // 同时发往 global 和所有区域
    Region(u8),                    // 发往指定区域
}
```

### SharedTransactionAccumulator 模式

跨模块共享表写入必须通过 `#[shared_table_reducer]` 宏包裹的 reducer 进行：

```rust
#[spacetimedb::reducer]
#[shared_table_reducer]          // 宏注入 RAII guard
pub fn empire_create(ctx: &ReducerContext, request: EmpireCreateRequest) -> Result<(), String> {
    // 内部自动初始化 SharedTransactionAccumulator
    EmpireState::insert_shared(ctx, empire, InterModuleDestination::GlobalAndAllOtherRegions);
    // reducer 结束时 drop() 自动 flush 所有挂起的跨模块消息
    Ok(())
}
```

**内部机制：**

```
#[shared_table_reducer] 宏生成代码（简化）:
{
    let __acc = SharedTransactionAccumulator { ctx };
    __acc.begin_shared_transaction();  // 初始化线程局部累积器
    // ... reducer 原始代码 ...
}  // __acc.drop() 在此调用 send_shared_transaction()
```

**线程局部累积器：**

```rust
thread_local! {
    static TABLE_UPDATES_GLOBAL: RefCell<InterModuleAccumulator> = ...;
    static TABLE_UPDATES_OTHER_REGIONS: RefCell<InterModuleAccumulator> = ...;
    static DELAYED_MESSAGES: RefCell<Vec<(MessageContentsV2, InterModuleDestination)>> = ...;
}
```

### 所有共享表及同步操作（InterModuleTableUpdates）

以下表被 `InterModuleTableUpdates` 结构体追踪，每个表有对应的 `XxxOp` 枚举（`Insert(T)` / `Delete(T)`）：

```
blocked_identity          building_nickname_state     building_state
claim_lowercase_name_state  claim_member_state        claim_state
empire_chunk_state        empire_node_siege_state      empire_node_state
empire_player_data_state  empire_rank_state            empire_settlement_state
empire_state              identity_role                location_state
player_housing_state      player_report_state          region_connection_info
region_control_info       region_population_info       region_sign_in_parameters
user_authentication_state  user_moderation_state       user_state
```

### 完整消息类型列表（MessageContentsV2，38 种）

| 类别 | 消息类型 |
|------|---------|
| 表同步 | `TableUpdate(InterModuleTableUpdates)` |
| 玩家迁移 | `TransferPlayerRequest`, `TransferPlayerHousingRequest` |
| 玩家管理 | `PlayerCreateRequest`, `UserUpdateRegionRequest`, `OnPlayerNameSetRequest`, `OnRegionPlayerCreated` |
| 登录控制 | `SignPlayerOut`, `PlayerSkipQueue` |
| 领地/定居点 | `ClaimCreateEmpireSettlementState`, `OnClaimMembersChanged`, `ClaimSetName` |
| 帝国管理 | `EmpireCreate`, `DeleteEmpire`, `EmpireClaimJoin`, `EmpireCreateBuilding`, `GlobalDeleteEmpireBuilding`, `OnEmpireBuildingDeleted` |
| 帝国操作 | `EmpireResupplyNode`, `EmpireDonateItem`, `EmpireAddCurrency`, `EmpireCollectHexiteCapsule`, `EmpireQueueSupplies`, `EmpireWithdrawItem` |
| 帝国人事 | `OnPlayerJoinedEmpire`, `OnPlayerLeftEmpire`, `EmpireUpdateEmperorCrown`, `EmpireRemoveCrown` |
| 攻城系统 | `EmpireStartSiege`, `EmpireSiegeAddSupplies`, `RegionDestroySiegeEngine` |
| 管理运营 | `AdminBroadcastMessage`, `GrantHubItem`, `ReplaceIdentity`, `RestoreSkills`, `NpcPlaceWatchtowers` |
| 部署物 | `RecoverDeployable`, `OnDeployableRecovered` |

### 消息去重机制

```rust
// 接收方追踪已处理消息
InterModuleMessageCounter { sender_module_id: u8, last_processed_message_id: u64 }

// 处理逻辑
if msg.id <= counter.last_processed_message_id {
    return; // 跳过重复消息
}
```

---

## 10. 世界生成系统

### 生成入口

```rust
// 位于 src/lib.rs 的 initialize reducer 中
pub fn generate_world(ctx: &ReducerContext, world_definition: WorldGenWorldDefinition) {
    if world_loaded(ctx) { return; }
    
    let generated_graph = WorldGraph::new(ctx, &mut world_definition);  // Step 1
    let generated_world = world_generator::generate(&world_definition, &generated_graph); // Step 2
    commit_generated_world(ctx, generated_world);  // Step 3
}
```

### 三阶段管线

```
WorldGenWorldDefinition（外部配置/编辑器输入）
         ↓
WorldGraph::new()
  ├── 计算地形类型（陆地/海洋/湖泊）
  ├── BFS 距离场（到海洋距离、到生物群落距离）
  ├── 计算高度（land_curve + 山脉 + 生物群落噪声）
  ├── 计算水位（海平面、湖泊洪泛）
  ├── 生成湖泊（噪声阈值法）
  ├── 生成河流（最小生成树 + A* 寻路）
  └── 放置资源与建筑
         ↓
world_generator::generate()
  ├── 地形节点 → TerrainChunkState（32×32格 = 1个区块）
  ├── 资源节点 → ResourceState + LocationState
  └── 建筑节点 → BuildingState + LocationState
         ↓
commit_generated_world()  →  写入 SpacetimeDB 数据库
```

### WorldDefinition 配置结构

```rust
pub struct WorldDefinition {
    pub size: Vector2Int,                       // 世界尺寸（单位：区块）
    pub land_curve: AnimationCurve,             // 高度曲线（基于距海距离）
    pub noise_influence: f32,                   // 噪声影响强度 [0..1]
    pub sea_level: i16,                         // 海平面高度 [1..128]
    pub world_map: WorldMapDefinition,          // 大陆轮廓（噪声定义）
    pub biomes_map: BiomesMapDefinition,        // 生物群落分布图
    pub mountains_map: MountainsMapDefinition,  // 山脉峰定义
    pub buildings_map: Vec<BuildingDetails>,    // 预置建筑/废墟列表
    pub resources_map: ResourcesMapDefinition,  // 资源分布配置
}
```

### 高度生成算法（多层叠加）

```
最终高度 = land_curve_contribution
         + mountain_contribution（如果在山脉范围内）
         + biome_noise_contribution（多噪声层叠加）
```

**生物群落噪声层（每层独立配置）：**
```rust
pub struct NoiseBasedElevationLayer {
    pub noise: NoiseSpecs,          // 种子、缩放、八度、持久性、空位率
    pub threshold: f32,             // 激活阈值（低于此值忽略此层）
    pub range: Vector2,             // 高度贡献范围 [min, max]
    pub blending_mode: BlendingMode // Add（叠加）或 Override（覆盖）
}
```

### 噪声算法

| 算法 | 用途 | 位置 |
|------|------|------|
| OpenSimplex 2D | 地形基础噪声 | `open_simplex_noise.rs` |
| 多倍频（Octave）噪声 | 高度细节层 | `noise_helper.rs` |

**NoiseSpecs：**
```rust
pub struct NoiseSpecs {
    pub seed: i32,          // 随机种子（决定世界唯一性）
    pub scale: f32,         // 频率缩放
    pub octaves: i32,       // 倍频数（越多越细腻）
    pub persistance: f32,   // 每倍频幅度衰减率
    pub lacunarity: f32,    // 每倍频频率倍增率
    pub offset: Vector2,    // 世界空间偏移（种子派生）
}
```

### 生物群落系统

生物群落图使用**二次螺旋坐标编码**将世界空间 x/z 映射到生物群落索引数组：

```rust
fn get_pixel_index(&self, coordinates: Vector2Int) -> Option<usize> {
    let (x, y) = (coordinates.x, coordinates.y);
    if x >= y { Some((x*x + y) as usize) }          // 螺旋编码
    else { Some((y*y + 2*y + 2 - x) as usize) }
}
```

每个 `BiomeDefinition` 包含：
- `distance_to_sea_curve` — 根据距海距离调整高度
- `transition_length` — 生物群落边界混合距离
- `noise_based_elevation_layers` — 多层噪声叠加高度
- `lake_noise_specs` / `lake_noise_threshold` — 湖泊生成参数
- `river_generation_settings` — 河流生成参数
- `terracing: bool` — 是否启用梯田地形

### 河流生成（6步管线）

1. **湖泊边界检测** — 识别湖泊边缘节点，按连通性分组
2. **路径候选生成** — 枚举所有湖泊对，计算路径成本
3. **最小生成树** — 用 Kruskal 算法选择最优河流网络
4. **A\* 寻路** — 为每条河流计算实际地形路径
5. **高度应用** — 线性插值源→目标高度，应用 `depth_curve` 截面轮廓
6. **侵蚀效果** — 根据 `erosion` 参数使河道下切地形

### 资源生成算法

```
对每种资源类型:
  对每个适用生物群落:
    对每个实体图节点:
      1. 检查生物群落归属（biomes_multipliers > 0）
      2. 检查高度范围（land_elevation_range 或 water_depth_range）
      3. 计算噪声值，与 noise_threshold [min,max] 对比
      4. 随机概率测试（基于 noise 值与 chance 参数）
      5. 检查占地面积合法性（rotation 穷举 6 方向）
      6. 全部通过 → 放置资源节点
```

### 区块系统

世界被划分为 **32×32 小地块** 的区块（TerrainChunkState），每个区块存储：
- 所有 32×32 格的高度值和地形类型
- 区块坐标（chunk_x = tile_x / 32）

---

## 11. 自定义过程宏（bitcraft-macro）

**文件：** `packages/game/bitcraft-macro/src/lib.rs`

### #[shared_table]

**用途：** 为需要跨模块同步的表自动生成三个方法。

**生成代码：**
```rust
impl TableName {
    pub fn insert_shared(ctx: &ReducerContext, val: TableName, destination: InterModuleDestination) {
        ctx.db.table_name().insert(val.clone());
        crate::inter_module::add_global_table_update(|u| u.table_name_push_insert(val.clone()));
        // 根据 destination 路由到 global 和/或其他区域
    }
    pub fn delete_shared(ctx: &ReducerContext, val: TableName, destination: InterModuleDestination) {
        ctx.db.table_name().primary_key().delete(val.primary_key);
        // 同步删除到目标模块
    }
    pub fn update_shared(ctx: &ReducerContext, val: TableName, destination: InterModuleDestination) {
        Self::delete_shared(ctx, val.clone(), destination);
        Self::insert_shared(ctx, val, destination);
    }
}
```

**参数变体：**
- `#[shared_table]` — 基本共享
- `#[shared_table(public_region)]` — 仅对区域公开
- `#[shared_table(public_global)]` — 仅对全局公开

### #[shared_table_reducer]

**用途：** 在 reducer 开头注入 `SharedTransactionAccumulator` RAII guard，确保 reducer 结束时所有共享表操作被 flush 为跨模块消息。

**生成代码（注入在 reducer 函数体最开始）：**
```rust
{
    let __shared_transaction_accumulator =
        crate::inter_module::SharedTransactionAccumulator { ctx };
    __shared_transaction_accumulator.begin_shared_transaction();
}
```

> ⚠️ **规则：** 任何调用 `*_shared()` 方法的 reducer **必须**带有此宏，否则共享表操作不会被发送到其他模块。

### #[feature_gate] / #[feature_gate("category")]

**用途：** 动态运行时功能开关，管理员可通过修改 `GatedFeatures` 表禁用特定 reducer 或功能类别。

**生成代码（注入在 reducer 函数体最开始）：**
```rust
{
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        // 检查 reducer 级开关
        let key = format!("reducer:{reducer_name}");
        if gated_features(&ctx.db).feature().find(&key).is_some() {
            return Err("This functionality is currently disabled".into());
        }
        // 如果指定了 category，还检查 category 级开关
        let cat_key = format!("category:{category_name}");
        if gated_features(&ctx.db).feature().find(&cat_key).is_some() {
            return Err("This functionality is currently disabled".into());
        }
    }
}
```

GM 及以上角色**绕过**所有功能门控。

### #[event_table(name = table_name)]

**用途：** 从普通 struct 生成完整的客户端事件定时表基础设施。

**输入：**
```rust
#[event_table(name = skill_cooldown)]
pub struct SkillCooldownEvent {
    pub player_id: u64,
    pub skill_id: i32,
}
```

**生成输出：**
```rust
#[spacetimedb::table(name = skill_cooldown, public, scheduled(skill_cooldown_reducer, at = scheduled_at))]
pub struct SkillCooldownEvent {
    #[primary_key] #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: spacetimedb::ScheduleAt,
    pub player_id: u64,
    pub skill_id: i32,
}

impl SkillCooldownEvent {
    pub fn new_event(ctx: &ReducerContext, player_id: u64, skill_id: i32) {
        let val = SkillCooldownEvent {
            scheduled_id: 0,
            scheduled_at: ctx.timestamp.into(),
            player_id, skill_id,
        };
        spacetimedb::Table::insert(skill_cooldown::table(ctx), val);
    }
}

#[spacetimedb::reducer]
fn skill_cooldown_reducer(_ctx: &ReducerContext, _timer: SkillCooldownEvent) {}
```

### #[static_data_staging_table(name)]

**用途：** 为静态数据表生成 `staged_*` 暂存表，用于批量导入前的数据验证。

**生成代码：**
```rust
// 在原始 struct 上追加属性
#[spacetimedb::table(name = staged_entity_definition)]
pub struct EntityDefinition { /* original fields */ }
```

暂存表由 `build_shared.rs` 的 `build_static_data_staging_tables()` 自动发现，并生成对应的 `stage_*` reducer 和 `validate_staged_data()` 函数。

### #[custom_inter_module_insert]

**用途：** 标记需要自定义插入逻辑的共享表，指示 `build_shared.rs` 生成调用 `inter_module_insert()` 而非直接 `insert()`。

---

## 12. 构建系统（Build System）

### 自动生成文件一览（勿手动编辑）

| 文件路径 | 生成函数 | 说明 |
|----------|---------|------|
| `src/game/handlers/*/mod.rs` | `build_all_handlers_mods()` | 所有 handler 域的模块声明 |
| `src/inter_module/_autogen.rs` | `build_shared_tables()` | 共享表 Op 枚举 + InterModuleTableUpdates 结构 |
| `src/game/autogen/_delete_entity.rs` | `build_gamestate_operations()` | `delete_entity()` / `clear_entity()` 函数 |
| `src/game/discovery/autogen/_discovery.rs` | `build_knowledge()` | 知识系统的所有 has/discover/acquire 方法 |
| `src/utils/version.rs` | `build_version_reducer()` | `current_version()` reducer（嵌入 git hash） |
| `src/game/static_data/autogen/` | `build_static_data_staging_tables()` | 暂存 reducer + 验证函数 |

### build.rs 执行流程

```rust
fn main() {
    build_shared::main_shared();  // 执行所有共享构建任务
    build_knowledge();             // 生成知识系统代码
}
```

`main_shared()` 依次执行：
1. `build_version_reducer()` — 读取 `git rev-parse HEAD` 嵌入版本
2. `build_all_handlers_mods()` — 扫描 handlers/ 目录，生成所有 mod.rs
3. `build_gamestate_operations()` — 扫描 components.rs，生成实体删除函数
4. `build_shared_tables()` — 扫描所有 messages/*.rs，生成共享表同步代码
5. `build_static_data_staging_tables()` — 生成静态数据暂存基础设施

### build_gamestate_operations 详解

扫描 `src/messages/components.rs` 中的所有表定义，收集：
- 有 `entity_id: u64` 字段的表 → `clear_entity()` 函数
- 同时有 `#[delete]` 属性的表 → `delete_entity()` 函数

```rust
// 生成的代码结构
pub fn delete_entity(ctx: &ReducerContext, entity_id: u64) {
    // 仅有 #[delete] 标记的表
    ctx.db.player_state().entity_id().delete(entity_id);
    ctx.db.health_state().entity_id().delete(entity_id);
    // ...
}

pub fn clear_entity(ctx: &ReducerContext, entity_id: u64) {
    // 所有有 entity_id 字段的表
    ctx.db.player_state().entity_id().delete(entity_id);
    ctx.db.knowledge_battle_action_state().entity_id().delete(entity_id);
    // ...（覆盖更多表）
}
```

### build_knowledge 详解

扫描 `src/messages/components.rs` 中带有特殊属性的表：

| 属性 | 知识类型 | 生成方法 |
|------|---------|---------|
| `#[knowledge]` | 通用知识 | `has_discovered_X()`, `discover_X()`, `acquire_X()` |
| `#[knowledge_entity]` | 实体知识（有位置） | 同上 + 实体绑定 |
| `#[knowledge_location]` | 地点知识 | 同上 + 坐标绑定 |
| `#[knowledge_recipe]` | 配方知识 | 同上 + 配方绑定 |
| `#[achievement]` | 成就 | 触发 `AchievementDesc::evaluate_all()` |

生成的 `Discovery` 对象通过哈希值追踪变更，只在内容修改时才写回数据库（避免无效写入）。

---

## 13. 静态数据导入系统

### 静态数据 vs 游戏状态

| 类型 | 命名规范 | 位置 | 可变性 |
|------|---------|------|-------|
| 静态数据 | `*_desc` 表（如 `building_desc`） | `src/messages/static_data.rs` | 只读，启动时导入 |
| 游戏状态 | `*_state` 表（如 `building_state`） | `src/messages/components.rs` | 可变，实时更新 |

### 导入流程

```
编辑器/工具生成数据
        ↓
Admin 调用 stage_xxx() reducer（批量）
        ↓
数据存入 staged_xxx 暂存表
        ↓
Admin 调用 import_xxx() reducer
  ├── validate_staged_data() 验证非空
  ├── 清空旧的 *_desc 表
  ├── 将 staged_xxx 内容写入正式 *_desc 表
  └── 调用 clear_staged_static_data() 清理暂存
```

### 关键静态数据表

| 表名 | 说明 |
|------|------|
| `building_desc` | 建筑类型（尺寸、占地、功能、制作选项） |
| `resource_desc` | 资源类型（采集产物、工具要求、刷新参数） |
| `item_desc` | 物品定义（类型、重量、堆叠上限） |
| `craft_recipe_desc` | 制作配方（材料、产物、制作时间） |
| `enemy_desc` | 敌人类型（血量、伤害、AI 参数、掉落表） |
| `parameters_desc` | 全局游戏参数（Agent tick 频率、各种上限） |
| `loot_table_desc` | 战利品掉落表（概率权重） |
| `collectible_desc` | 收藏品定义 |
| `claim_tech_desc` | 领地科技树定义 |
| `deployable_desc` | 可部署物定义 |

---

## 14. 权限与安全模型

### Role 枚举（从低到高）

```rust
#[repr(i32)]
pub enum Role {
    Player    = 0,  // 普通玩家
    Partner   = 1,  // 合作伙伴
    SkipQueue = 2,  // 可跳过登录队列
    Mod       = 3,  // 审核员（举报处理）
    Gm        = 4,  // 游戏管理员（GM）
    Admin     = 5,  // 服务器管理员
    Relay     = 6,  // 中继服务账号
}
```

### 权限检查（层级式）

```rust
pub fn has_role(ctx: &ReducerContext, identity: &Identity, role: Role) -> bool {
    match ctx.db.config().version().find(&0) {
        Some(config) if config.env == "dev" => return true, // dev 环境不鉴权
        None => return true,
        _ => {}
    }
    match ctx.db.identity_role().identity().find(identity) {
        Some(entry) if entry.role as i32 >= role as i32 => true, // 层级比较
        _ => false,
    }
}
```

Admin（5）可以执行要求 Mod（3）的操作，因为 5 >= 3。

### IdentityRole 表

```rust
#[shared_table]  // 全局所有，复制到所有区域
#[spacetimedb::table(name = identity_role, public)]
pub struct IdentityRole {
    #[primary_key]
    pub identity: Identity,  // SpacetimeDB 用户身份（区块链地址）
    pub role: Role,
}
```

### 领地权限层级

领地内的操作权限通过 `ClaimMemberState` 管理：

| 权限字段 | 允许操作 |
|---------|---------|
| `inventory_permission` | 访问领地内的存储箱 |
| `build_permission` | 在领地内建造/拆除 |
| `officer_permission` | 管理成员、设置 |
| `co_owner_permission` | 领地所有者级别操作 |

帝国权限通过 `EmpireRankState.permissions: Vec<bool>` 管理（按 `EmpirePermission` 枚举索引）。

### ServerIdentity 验证

服务端内部 reducer（不由玩家调用）使用：
```rust
ServerIdentity::validate_server_or_admin(&ctx)?;
```

---

## 15. Global 模块完整分析

### 模块初始化

```
initialize() reducer:
  ├── 创建 IdentityRole(Admin) — 授予调用者管理员权限
  ├── 创建 AdminBroadcast (version=0) — 全局公告系统
  ├── 创建 Globals — 实体计数器、维度计数器、区域索引
  └── 创建 Config — 环境(dev/prod)、agents_enabled 标志
```

### 帝国创建流程（跨模块协作）

```
[Game Region]  empire_create reducer
  ├── 验证玩家拥有领地建筑
  ├── 验证玩家未加入其他帝国
  ├── 验证帝国名唯一
  ├── 从玩家扣除 empire_shard_cost 碎片
  └── 发送 EmpireCreateMsg → Global

[Global Module]  process_inter_module_message → empire_create.rs
  ├── 验证帝国名全球唯一（含保留名）
  ├── 创建 EmpireState 实体
  ├── 向所有区域复制 EmpireState（insert_shared → AllOtherRegions）
  ├── 创建 EmpireEmblemState（帝国徽章）
  ├── 创建 EmpireDirectiveState（帝国公告）
  ├── 创建默认 EmpireRankState（5个等级）
  ├── 创建 EmpirePlayerDataState（创始人=皇帝，rank=0）
  ├── 更新 EmpireSettlementState.empire_entity_id
  ├── 调用 update_empire_upkeep()
  └── 调用 update_crown_status()
```

### 攻城系统详解

**发起攻城（跨模块）：**
```
[Game Region]  empire_start_siege reducer
  └── 发送 EmpireStartSiegeMsg → Global

[Global Module]  empire_start_siege.rs
  ├── 创建 EmpireNodeSiegeState（energy = 初始供给）
  └── 向所有区域广播
```

**Empire Siege Agent 循环（全局模块）：**
```
每个 Tick（empire_siege_tick_millis 毫秒）:
  对每个 active=true 的攻城实例:
    cost = cost_for_next_tick(tick_secs, raise_pct, start_time, now)

    如果 cost > siege.energy（攻城方供给不足）:
      → 攻城失败，end_siege(防守方胜利)
    否则:
      siege.energy -= cost
      如果 defender_node.energy < drain:
        node.energy = 0
        → 攻城成功，end_siege(攻击方胜利)，转移领土控制权
```

**成本公式（指数增长）：**
`cost = tick_secs * raise_pct^(elapsed_seconds / tick_secs)`

### 玩家跨区迁移

```
[Source Region]  player_region_crossover reducer
  ├── 收集玩家所有状态（TransferPlayerMsgV2）
  ├── 删除本区域的玩家实体
  └── 发送 TransferPlayerRequest → Target Region（经 Global 路由）

[Global Module]  user_update_region.rs
  ├── 更新 UserRegionState.region_id
  └── 转发消息到目标区域

[Target Region]
  ├── 重建玩家所有组件
  ├── 关联住宅/传送门系统
  └── 通知玩家已到达新区域
```

**迁移的状态内容（TransferPlayerMsgV2）：**
PlayerState, UserState, HealthState, StaminaState, ExperienceState, EquipmentState, InventoryState, VaultState, 所有 KnowledgeXxxState, AbilityState, ActionBarState, OnboardingState, PlayerSettingsState, 可选 PlayerHousingState

### Global 模块关键表

**玩家全局状态：**

| 表名 | 说明 |
|------|------|
| `UserRegionState` | identity → 所属区域 ID 映射 |
| `PlayerShardState` | 玩家碎片货币余额 |
| `PlayerUsernameState` | 用户名（全局唯一） |
| `PreviousPlayerUsernameState` | 历史名称保留（防抢占） |

**社交系统：**

| 表名 | 说明 |
|------|------|
| `FriendsState` | 好友关系 |
| `BlockedPlayerState` | 屏蔽列表 |
| `ChatChannelState` | 聊天频道 |
| `ChatMessageState` | 聊天历史 |

**审核系统：**

| 表名 | 说明 |
|------|------|
| `UserModerationState` | 封禁/禁言记录（有效期限）。全局所有，复制到所有区域 |
| `ModerationActionLogState` | GM 操作审计日志 |

**审核执行逻辑：**
- 创建封禁策略 → 如果是 BlockLogin，立即通过 `SignPlayerOut` 消息踢出玩家
- 创建禁言策略 → 删除该玩家所有 `ChatMessageState`
- 玩家连接时 → 检查 `UserModerationState`，有效封禁则拒绝连接

### Global 模块 Handler 域概览

| 域 | 文件 | 关键功能 |
|----|------|---------|
| `empires/` | `empires.rs` | 帝国创建、成员管理、等级设置 |
| `player/` | `sign_in.rs`, `player_set_name.rs` 等 | 玩家认证、命名、日常签到 |
| `admin/` | `admin_grant_shards.rs` 等 | 管理员操作（授予碎片、广播、改名） |
| `gm/` | `user_moderation.rs` | 封禁/禁言/撤销审核 |

---

## 16. 构建、发布、客户端绑定命令速查

### 构建

```bash
# 在 game 模块目录下构建
cd BitCraftServer/packages/game
spacetime build

# 在 global_module 目录下构建
cd BitCraftServer/packages/global_module
spacetime build
```

### 发布

```bash
# 标准发布（不重置数据）
spacetime publish <module-name> -s <server-url>

# 强制发布（Schema 不兼容时重置数据）
spacetime publish <module-name> -s <server-url> -c -y
```

### 客户端绑定生成

```bash
# 生成 C# 绑定（Unity 用）
./generate-client-files.sh

# 生成 TypeScript 绑定
./generate-client-files.sh -l ts
```

### 代码格式化

```bash
# 格式化（行宽 140，见 rustfmt.toml）
rustfmt --edition 2021 src/**/*.rs
```

### CI/CD（GitHub Actions）

项目包含 `.github/workflows/upload-module.yml`，在代码推送时自动构建并发布到测试服务器。

### 版本确认

```bash
# 调用 current_version reducer 确认服务器版本（返回 git hash 到日志）
spacetime call <module> current_version
```

---

## 附录：核心架构原则总结

1. **SpacetimeDB = 唯一真相来源** — 所有状态在 SpacetimeDB 表中，没有进程内状态。
2. **Reducer = 唯一变更入口** — 所有业务逻辑在 Rust reducer 中执行，失败则全量回滚。
3. **ECS 无对象** — 实体 = u64 ID，属性 = 多个表中的行，通过共享 entity_id 关联。
4. **异步跨模块** — 两个模块通过消息表异步通信，永不同步 RPC。
5. **共享表 = 唯一同步机制** — 跨模块数据通过 `#[shared_table]` + `SharedTransactionAccumulator` 批量同步。
6. **构建时代码生成** — 模块声明、实体删除、知识系统、版本嵌入全部由 build.rs 自动生成，勿手动修改。
7. **参数驱动调速** — 所有 Agent tick 频率来自 `parameters_desc` 静态数据，运行时可热更新。
8. **维度不建索引** — `LocationState.dimension` 字段故意不加索引，使用 `dimension_filter()` 辅助函数。

---

*文档由 GitHub Copilot 通过深度源码分析自动生成，基于 BitCraftPublic 仓库 master 分支*