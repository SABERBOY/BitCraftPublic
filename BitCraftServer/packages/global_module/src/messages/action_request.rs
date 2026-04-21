use spacetimedb::{Identity, SpacetimeType};

use crate::messages::components::PermissionGroup;
use crate::messages::game_util::*;
use crate::messages::util::*;
use crate::ChatChannel;
use crate::EntityType;

use super::components::Permission;
use super::components::UserModerationPolicy;
use super::static_data::EnemyType;

#[derive(SpacetimeType, Clone)]
pub struct PlayerMoveRequest {
    pub timestamp: u64,
    pub destination: Option<OffsetCoordinatesFloat>,
    pub origin: Option<OffsetCoordinatesFloat>,
    pub duration: f32,
    pub move_type: i32,
    pub running: bool,
}

#[derive(SpacetimeType)]
pub struct EnemyMoveRequest {
    pub entity_id: u64,
    pub chunk_index: u64,
    pub origin: OffsetCoordinatesFloat,
    pub destination: OffsetCoordinatesFloat,
    pub duration: f32,
    pub overlap_prevention: bool,
}

#[derive(SpacetimeType)]
pub struct EnemySpawnRequest {
    pub enemy_type: EnemyType,
    pub herd_entity_id: u64,
    pub coord: OffsetCoordinatesSmallMessage,
}

#[derive(SpacetimeType)]
pub struct EnemySetHealthRequest {
    pub entity_id: u64,
    pub health: i32,
}

#[derive(SpacetimeType)]
pub struct EnemyClearAggroRequest {
    pub entity_id: u64,
    pub aggro_entity_id: Option<u64>,
}

#[derive(SpacetimeType)]
pub struct ServerResetMobileEntityPositionRequest {
    pub owner_entity_id: u64,
    pub position: Option<OffsetCoordinatesFloat>,
}

#[derive(SpacetimeType)]
pub struct PlayerClimbRequest {
    pub destination: OffsetCoordinatesFloat,
    pub origin: OffsetCoordinatesFloat,
    pub timestamp: u64,
    pub climb_type: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerSetDefaultDeployableRequest {
    pub deployable_collectible_id: i32,
}

#[derive(SpacetimeType, Copy, Clone)]
pub struct PlayerExtractRequest {
    pub recipe_id: i32,
    pub target_entity_id: u64,
    pub timestamp: u64,
    pub clear_from_claim: bool, // no xp, no requirements, no yield
}

#[derive(SpacetimeType)]
pub struct PlayerPassiveCraftStartRequest {
    pub recipe_id: i32,
    pub building_entity_id: u64,
    pub inventory_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerPassiveCraftQueueRequest {
    pub recipe_id: i32,
    pub building_entity_id: u64,
}

#[derive(SpacetimeType, Copy, Clone, Debug)]
pub struct PlayerCraftInitiateRequest {
    pub recipe_id: i32,
    pub building_entity_id: u64,
    pub count: i32,
    pub timestamp: u64,
    pub is_public: bool,
}

#[derive(SpacetimeType, Copy, Clone, Debug)]
pub struct PlayerCraftContinueRequest {
    pub progressive_action_entity_id: u64,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerCraftCollectRequest {
    pub pocket_id: u64,
    pub recipe_id: i32, //Needed by clients
}

#[derive(SpacetimeType)]
pub struct PlayerCraftCollectAllRequest {
    pub building_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerCraftCancelRequest {
    pub pocket_id: u64,
}

#[derive(SpacetimeType)]
pub struct AgentCraftUnlockRequest {
    pub pocket_id: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerProjectSitePlaceRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub construction_recipe_id: i32,
    pub resource_placement_recipe_id: i32,
    pub facing_direction: i32,
}

#[derive(SpacetimeType, Debug)]
pub struct CheatCompendiumItemPlaceRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub item_id: i32,
    pub facing_direction: i32,
}

#[derive(SpacetimeType, Debug)]
pub struct CheatCompendiumEnemyPlaceRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub enemy_type: EnemyType,
}

#[derive(SpacetimeType)]
pub struct PlayerProjectSiteAddMaterialsRequest {
    pub owner_entity_id: u64,
    pub pockets: Vec<PocketKey>,
}

#[derive(SpacetimeType)]
pub struct PlayerProjectSiteCancelRequest {
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType, Copy, Clone)]
pub struct PlayerProjectSiteAdvanceProjectRequest {
    pub owner_entity_id: u64,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerSleepRequest {
    pub dummy: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerSetHomeRequest {
    pub target_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerSitRequest {
    pub dummy: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerItemDropRequest {
    pub pocket: PocketKey,
}

#[derive(SpacetimeType)]
pub struct PlayerItemStackMoveRequest {
    pub from_pocket: PocketKey, // TODO: This will replace PickupCargo, WithdrawCargo, PickupItem, WithdrawItem
    pub to_pocket: PocketKey,
    pub quantity: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerItemStackMoveAllRequest {
    pub from_pocket: PocketKey,
    pub to_pocket: PocketKey,
}

#[derive(SpacetimeType)]
pub struct PlayerItemStackSplitRequest {
    pub from_pocket: PocketKey,
    pub item_id: i32,
    pub new_stack_count: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeInitiateSessionRequest {
    pub acceptor_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeAcceptSessionRequest {
    pub session_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeDeclineSessionRequest {
    pub session_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeAddItemRequest {
    pub session_entity_id: u64,
    pub pocket_index: i32,
    pub inventory_pocket_index: i32,
    pub inventory_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeRemoveItemRequest {
    pub session_entity_id: u64,
    pub pocket_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeSwapPocketsRequest {
    pub session_entity_id: u64,
    pub from_index: i32,
    pub to_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeAcceptRequest {
    pub session_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerTradeDeclineRequest {
    pub session_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerBarterStallOrderAccept {
    pub trade_order_entity_id: u64,
    pub shop_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerSignInRequest {
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerChatPostMessageRequest {
    pub text: String,
    pub channel_id: ChatChannel, // Only used for regional messages
    pub target_id: u64,          // Only used for targeted messages
    pub language_code: String,
}

#[derive(SpacetimeType)]
pub struct PlayerSetNameRequest {
    pub username: String,
}

#[derive(SpacetimeType)]
pub struct PlayerCargoPickUpRequest {
    pub cargo_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerPocketSwapContentsRequest {
    pub from_pocket: PocketKey,
    pub to_pocket: PocketKey,
    pub quantity: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTeleportHomeRequest {
    pub dummy: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerTeleportWaystoneRequest {
    pub entity_id_from: u64,
    pub entity_id_to: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerDismissAlertRequest {
    pub alert_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct CheatExperienceGrantRequest {
    pub owner_entity_id: u64,
    pub skill_id: i32,
    pub amount: i32,
}

#[derive(SpacetimeType)]
pub struct CheatShardsGrantRequest {
    pub owner_entity_id: u64,
    pub shards: u32,
}

#[derive(SpacetimeType)]
pub struct CheatCargoGrantRequest {
    pub owner_entity_id: u64,
    pub cargo_id: i32,
}

#[derive(SpacetimeType, Debug)]
pub struct CheatWarpRequest {
    pub location: OffsetCoordinatesLargeMessage,
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerDeployableMountRequest {
    pub deployable_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerDeployableDismountRequest {
    pub deployable_entity_id: u64,
    pub player_entity_id: u64, // for server request when storing a populated deployable
    pub coordinates: Option<OffsetCoordinatesFloat>,
    pub deployable_coordinates: Option<OffsetCoordinatesFloat>,
    pub skip_deployable_icon: bool,
}

#[derive(SpacetimeType)]
pub struct CargoSpawnRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub cargo_id: i32,
    pub direction: i32,
    pub initiator_entity_id: u64,
    pub warn_entity_id: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct ServerCargoDespawnCallback {
    pub cargo_entity_id: u64,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct EntityAttackRequest {
    pub attacker_entity_id: u64,
    pub defender_entity_id: u64,
    pub combat_action_id: i32,
    pub attacker_type: EntityType,
    pub defender_type: EntityType,
}

#[derive(SpacetimeType)]
pub struct NpcMoveRequest {
    pub npc_entity_id: u64,
    pub direction: i32,
    pub building_entity_id: u64,
    pub teleport_debug: Option<OffsetCoordinatesSmallMessage>,
}

#[derive(SpacetimeType)]
pub struct PlayerDeployableMoveRequest {
    pub deployable_entity_id: u64,
    pub timestamp: u64,
    pub destination: Option<OffsetCoordinatesFloat>,
    pub origin: Option<OffsetCoordinatesFloat>,
    pub duration: f32,
}

#[derive(SpacetimeType)]
pub struct AgentStaminaRegenRequest {
    pub owner_entity_id: u64,
    pub quantity: f32,
}

#[derive(SpacetimeType)]
pub struct AgentHungerRegenRequest {
    pub owner_entity_id: u64,
    pub quantity: f32,
}

#[derive(SpacetimeType)]
pub struct AgentHealthRegenRequest {
    pub owner_entity_id: u64,
    pub quantity: f32,
}

#[derive(SpacetimeType)]
pub struct PlayerBuildingMoveRequest {
    pub building_entity_id: u64,
    pub new_coordinates: OffsetCoordinatesSmallMessage,
    pub facing_direction: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerQuestCompleteRequest {
    pub quest_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerBuildingDeconstructRequest {
    pub building_entity_id: u64,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerCompleteQuestTaskRequest {
    pub quest_index: i32,
    pub task_name: String,
}

#[derive(SpacetimeType)]
pub struct PlayerQuestClearStateRequest {
    pub target_entity_id: u64,
    pub quest_id: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerEquipmentAddRequest {
    pub from_pocket: Option<PocketKey>,
    pub preset_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerEquipmentRemoveRequest {
    pub slot: i32,
    pub to_pocket: Option<PocketKey>,
    pub preset_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerBuildingSetNicknameRequest {
    pub building_entity_id: u64,
    pub nickname: String,
}

#[derive(SpacetimeType)]
pub struct PlayerBuildingRepairRequest {
    pub building_entity_id: u64,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct ClaimResupplyRequest {
    pub building_entity_id: u64,
    pub from_pocket: PocketKey,
}

#[derive(SpacetimeType)]
pub struct ClaimSetPurchaseSupplyThresholdRequest {
    pub building_entity_id: u64,
    pub threshold: u32,
}

#[derive(SpacetimeType)]
pub struct ClaimSetPurchaseSupplyPriceRequest {
    pub building_entity_id: u64,
    pub purchase_price: f32,
}

#[derive(SpacetimeType)]
pub struct ClaimPurchaseSuppliesFromPlayerRequest {
    pub building_entity_id: u64,
    pub paid_supplies: i32,    // in case the amount of supplies to threshold changed since the acceptation
    pub price_per_supply: f32, // in case the price per supplies changed since the acceptation
}

#[derive(SpacetimeType)]
pub struct PlayerEatRequest {
    pub pocket_index: i32,
}

#[derive(SpacetimeType)]
pub struct RemoveFavoriteFriendRequest {
    pub player_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerScrollReadRequest {
    pub pocket_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerCollectibleActivateRequest {
    pub vault_index: i32,
    pub activated: bool,
}

#[derive(SpacetimeType)]
pub struct DeployableDeployRequest {
    pub vault_index: i32,
    pub coord: OffsetCoordinatesSmallMessage,
    pub direction: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerConvertCollectibleToDeedRequest {
    pub vault_index: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerConvertDeedToCollectibleRequest {
    pub pocket_index: i32,
}

#[derive(SpacetimeType)]
pub struct CheatDiscoverMapRequest {
    pub target_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct CheatTeleportFloatRequest {
    pub player_entity_id: u64,
    pub destination: Option<OffsetCoordinatesFloat>,
}

#[derive(SpacetimeType)]
pub struct CheatGrantKnowledgeRequest {
    pub target_entity_id: u64,
    pub also_learn: bool,
}

#[derive(SpacetimeType, Debug)]
pub struct CheatSetDebugAiStateRequest {
    pub entity_id: u64,
    pub target_entity_id: u64,
    pub target_position: OffsetCoordinatesFloat,
    pub current_position: OffsetCoordinatesFloat,
    pub current_destination: OffsetCoordinatesFloat,
    pub dp: f32,
}

#[derive(SpacetimeType, Debug)]
pub struct UserModerationCreateUserPolicyRequest {
    pub target_identity: Identity,
    pub user_moderation_policy: UserModerationPolicy,
    pub duration_ms: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct UserModerationSoftDeleteUserRequest {
    pub user_moderation_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct DeployableStoreRequest {
    pub deployable_entity_id: u64,
    pub remotely: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimTakeOwnershipRequest {
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimAddMemberRequest {
    pub player_name: String,
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimAddRecruitmentRequest {
    pub claim_entity_id: u64,
    pub stock: i32,
    pub required_skill_id: i32,
    pub required_skill_level: i32,
    pub required_approval: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimApplyForRecruitmentRequest {
    pub recruitment_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimRemoveRecruitmentRequest {
    pub recruitment_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimRemoveMemberRequest {
    pub player_entity_id: u64,
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimSetMemberPermissionsRequest {
    pub player_entity_id: u64,
    pub claim_entity_id: u64,
    pub inventory: bool,
    pub build: bool,
    pub officer: bool,
    pub co_owner: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimTransferOwnershipRequest {
    pub new_owner_entity_id: u64,
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimRenameRequest {
    pub claim_entity_id: u64,
    pub claim_name: String,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimTechLearnRequest {
    pub claim_entity_id: u64,
    pub tech_id: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimTechCancelRequest {
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimTechUnlockTechRequest {
    pub claim_entity_id: u64,
    pub tech_id: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimAddTileRequest {
    pub claim_entity_id: u64,
    pub coordinates: OffsetCoordinatesSmallMessage,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimRemoveTileRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimLeaveRequest {
    pub claim_entity_id: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerPavingPlaceTileRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub tile_type_id: i32,
    pub timestamp: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerPillarShapingPlaceRequest {
    pub coordinates: OffsetCoordinatesLargeMessage,
    pub pillar_type_id: i32,
    pub timestamp: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerPavingDestroyTileRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub timestamp: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerPillarShapingDestroyRequest {
    pub coordinates: OffsetCoordinatesLargeMessage,
    pub timestamp: u64,
}

#[derive(SpacetimeType, Debug)]
pub struct PlayerCompleteTaskRequest {
    pub task_entity_id: u64,
    pub npc_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimWithdrawFromTreasuryRequest {
    pub claim_entity_id: u64,
    pub amount: u32,
}

#[derive(SpacetimeType)]
pub struct PlayerClaimDepositToTreasuryRequest {
    pub claim_entity_id: u64,
    pub amount: u32,
}

#[derive(SpacetimeType, Copy, Clone)]
pub struct PlayerTerraformSetFinalTargetRequest {
    pub coordinates: OffsetCoordinatesLargeMessage,
    pub final_height_target: i16,
}

#[derive(SpacetimeType, Copy, Clone)]
pub struct PlayerTerraformRequest {
    pub coordinates: OffsetCoordinatesLargeMessage,
    pub start_new: bool,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerTerraformCancelRequest {
    pub coordinates: OffsetCoordinatesLargeMessage,
}

#[derive(SpacetimeType)]
pub struct PlayerBarterStallOrderCreateRequest {
    pub shop_entity_id: u64,
    pub remaining_stock: i32,
    pub offer_items: Vec<ItemStack>,
    pub required_items: Vec<ItemStack>,
}

#[derive(SpacetimeType)]
pub struct PlayerBarterStallOrderDeleteRequest {
    pub shop_entity_id: u64,
    pub trade_order_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct BarterStallSetMarketModeEnabledRequest {
    pub shop_entity_id: u64,
    pub enabled: bool,
}

#[derive(SpacetimeType, Debug)]
pub struct CheatToggleActiveCollectibleRequest {
    pub owner_entity_id: u64,
    pub item_deed_id: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerEmoteRequest {
    pub emote_id: i32,
    pub face: Option<OffsetCoordinatesSmallMessage>,
}

#[derive(SpacetimeType)]
pub struct TargetUpdateRequest {
    pub owner_entity_id: u64,
    pub target_entity_id: u64,
    pub generate_aggro: bool,
}

#[derive(SpacetimeType)]
pub struct ServerLootChestSpawnCallback {
    pub building_spawn_id: i32,
    pub building_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct ServerLootChestDespawnCallback {
    pub loot_chest_entity_id: u64,
    pub should_respawn: bool,
}

#[derive(SpacetimeType)]
pub struct EnemySpawnLootRequest {
    pub enemy_type: i32,
    pub player_entity_id: u64, // for server request when storing a populated deployable
    pub loot_coordinates: SmallHexTileMessage,
}

#[derive(SpacetimeType)]
pub struct CheatSpawnLootChestRequest {
    pub coordinates: OffsetCoordinatesSmallMessage,
    pub loot_chest_id: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerDroppedInventoryPickUpRequest {
    pub dropped_inventory_entity_id: u64,
    pub item_id: i32,
    pub pocket_index: i32,
    pub to_deployable: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerDiscoverEntitiesRequest {
    pub discovered_entities_id: Vec<u64>,
}

#[derive(SpacetimeType)]
pub struct PlayerAcquireKnowledgeFromEntitiesRequest {
    pub discovered_entities_id: Vec<u64>,
}

#[derive(SpacetimeType)]
pub struct PlayerItemConvertRequest {
    pub conversion_recipe_id: u32,
    pub location_context: u32,
    pub count: u32,
    pub timestamp: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerPortalEnterRequest {
    pub portal_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerPermissionEditRequest {
    pub ordained_entity_id: u64,
    pub allowed_entity_id: u64,
    pub permission: Option<Permission>,
    pub group: PermissionGroup,
}

#[derive(SpacetimeType)]
pub struct PlayerPausePlayTimerRequest {
    pub player_entity_id: u64,
    pub paused: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerTeleportToPlayerRequest {
    pub target_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerVoteAnswerRequest {
    pub vote_entity_id: u64,
    pub accept: bool,
}

#[derive(SpacetimeType)]
pub struct PlayerVoteConcludeRequest {
    pub vote_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerItemUseRequest {
    pub pocket_index: i32,
    pub arg_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct ServerInteriorSetCollapsedCallback {
    pub dimension_network_entity_id: u64,
    pub is_collapsed: bool,
}

#[derive(SpacetimeType, Copy, Clone)]
pub enum ServerTeleportReason {
    RuinCollapse,
    InteriorDeconstructed,
    PlayerDied,
    TeleportItem,
    PlayerHousingChangedLocation,
    PlayerHousingUnderMaintenance,
    PlayerHousingDeconstructed,
}

#[derive(SpacetimeType)]
pub struct ServerTeleportPlayerRequest {
    pub location: OffsetCoordinatesFloat,
    pub player_entity_id: u64,
    pub reason: ServerTeleportReason,
}

#[derive(SpacetimeType)]
pub struct ServerDestroyDimensionNetworkRequest {
    pub player_teleport_location: OffsetCoordinatesFloat,
    pub dimension_network_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct ServerEnemyDespawnRequest {
    pub entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerAchievementClaimRequest {
    pub achievement_id: i32,
}

#[derive(SpacetimeType)]
pub struct RentSetDailyRateRequest {
    pub rent_entity_id: u64,
    pub daily_rate: u32,
}

#[derive(SpacetimeType)]
pub struct RentAddTenantRequest {
    pub rent_entity_id: u64,
    pub tenant_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentRemoveTenantRequest {
    pub rent_entity_id: u64,
    pub tenant_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentTerminateRequest {
    pub rent_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentEvictRequest {
    pub rent_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentPurchaseRequest {
    pub rent_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentDepositCoinsRequest {
    pub rent_entity_id: u64,
    pub amount: u32,
}

#[derive(SpacetimeType)]
pub struct RentAddListingRequest {
    pub rent_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct RentUnlistRequest {
    pub rent_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct BuildingSetSignTextRequest {
    pub building_entity_id: u64,
    pub text: String,
}

#[derive(SpacetimeType)]
pub struct PlayerPostOrderRequest {
    pub building_entity_id: u64,
    pub item_id: i32,
    pub item_type: ItemType,
    pub max_unit_price: i32,
    pub quantity: i32,
    pub persist_order: bool, // if false, we limit the coins or materials to the amount of matching existing orders
    pub coins_spent: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerEditOrderRequest {
    pub building_entity_id: u64,
    pub order_entity_id: u64,
    pub coins: i32, // unit price
    pub quantity: i32,
}

#[derive(SpacetimeType)]
pub struct PlayerOrderCancelRequest {
    pub building_entity_id: u64,
    pub auction_listing_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerOrderCollectRequest {
    pub building_entity_id: u64,
    pub closed_listing_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerClosedListingCollectRequest {
    pub building_entity_id: u64,
    pub auction_listing_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerHousingEnterRequest {
    pub building_entity_id: u64,
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerHousingRequestAccessRequest {
    pub building_entity_id: u64,
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerHousingEvictPlayerRequest {
    pub building_entity_id: u64,
    pub owner_entity_id: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerRetrieveLostItemRequest {
    pub building_entity_id: u64,
    pub item_id: i32,
    pub is_cargo: bool,
    pub durability: i32,
    pub target_inventory_entity_id: u64,
    pub target_inventory_index: Option<i32>,
}

#[derive(SpacetimeType)]
pub struct ReportPlayerChatMessage {
    pub chat_message_id: u64,
    pub report_type: String,
    pub message: String,
}

#[derive(SpacetimeType)]
pub struct ReportPlayerMessage {
    pub player_entity_id: u64,
    pub report_type: String,
    pub message: String,
}

#[derive(SpacetimeType)]
pub struct CreatePlayerReportRequest {
    pub reported_player_entity_id: u64,
    pub reporter_entity_id: u64,
    pub chat_message_id: u64,
    pub report_type: String,
    pub message: String,
}

#[derive(SpacetimeType)]
pub struct ReportEntityMessage {
    pub entity_id: u64,
    pub report_type: String,
    pub message: String,
}

// TODO: Delete
#[derive(SpacetimeType)]
pub struct ReportPlayerDirectChatMessage {
    pub chat_message_id: u64,
    pub report_type: String,
    pub message: String,
}
