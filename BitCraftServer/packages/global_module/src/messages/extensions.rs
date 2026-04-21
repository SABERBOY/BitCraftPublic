use spacetimedb::{ReducerContext, Table};

use super::{
    components::{Biome, Permission, PermissionGroup},
    generic::{globals, region_sign_in_parameters, RegionSignInParameters},
    static_data::*,
};

impl BuildingInteractionLevel {
    pub fn to_enum(value: i32) -> BuildingInteractionLevel {
        unsafe { std::mem::transmute(value) }
    }
}

impl FootprintType {
    pub fn to_enum(value: i32) -> FootprintType {
        unsafe { std::mem::transmute(value) }
    }
}

impl InteriorSpawnType {
    pub fn to_enum(value: i32) -> InteriorSpawnType {
        unsafe { std::mem::transmute(value) }
    }
}

impl MovementType {
    pub fn to_enum(value: i32) -> MovementType {
        unsafe { std::mem::transmute(value) }
    }
}

impl BuffTypeDesc {
    pub fn filter_by_category<'a>(ctx: &'a ReducerContext, category: BuffCategory) -> impl Iterator<Item = Self> + 'a {
        let category = category as i32;
        ctx.db.buff_type_desc().iter().filter(move |a| a.category == category)
    }
}

impl NpcType {
    pub fn to_enum(value: i32) -> NpcType {
        unsafe { std::mem::transmute(value) }
    }
}

impl Permission {
    pub fn to_enum(value: i32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl PermissionGroup {
    pub fn to_enum(value: i32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl BuffCategory {
    pub fn has_only_one_buff(self) -> bool {
        match self {
            BuffCategory::None | BuffCategory::Generic => false,
            _ => true,
        }
    }
    pub fn to_enum(value: i32) -> BuffCategory {
        unsafe { std::mem::transmute(value) }
    }
}

impl BuffDesc {
    pub fn filter_by_buff_category<'a>(ctx: &'a ReducerContext, category: BuffCategory) -> impl Iterator<Item = Self> + 'a {
        let category = category as i32;
        ctx.db
            .buff_type_desc()
            .category()
            .filter(category)
            .map(|bt| bt.id)
            .flat_map(|t| ctx.db.buff_desc().buff_type_id().filter(t))
    }

    pub fn find_by_buff_category_single(ctx: &ReducerContext, category: BuffCategory) -> Option<Self> {
        if !category.has_only_one_buff() {
            panic!("Use `filter_by_buff_category` to filter by generic categories");
        }
        let category = category as i32;
        ctx.db
            .buff_type_desc()
            .category()
            .filter(category)
            .map(|bt| bt.id)
            .flat_map(|t| ctx.db.buff_desc().buff_type_id().filter(t))
            .next()
    }
}

impl EnemyAiParamsDesc {
    pub fn filter_by_enemy_type<'a>(ctx: &'a ReducerContext, enemy_type: &'a EnemyType) -> impl Iterator<Item = Self> + 'a {
        ctx.db.enemy_ai_params_desc().iter().filter(|a| a.enemy_type == *enemy_type)
    }
}

impl BuildingCategory {
    pub fn to_enum(value: i32) -> BuildingCategory {
        unsafe { std::mem::transmute(value) }
    }
}

impl Biome {
    pub fn to_enum(value: u8) -> Biome {
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl SkillType {
    pub fn to_enum(value: i32) -> Self {
        unsafe { std::mem::transmute(value as i32) }
    }
}

impl ExtractionRecipeDesc {
    pub fn get_skill_type(&self) -> Option<SkillType> {
        let skill_id = match self.experience_per_progress.first() {
            Some(exp) => exp.skill_id,
            None => 0,
        };
        if skill_id > SkillType::ANY as i32 {
            return Some(SkillType::to_enum(skill_id));
        }
        return None;
    }
}

impl CraftingRecipeDesc {
    pub fn get_skill_type(&self) -> Option<SkillType> {
        let skill_id = match self.experience_per_progress.first() {
            Some(exp) => exp.skill_id,
            None => 0,
        };
        if skill_id > SkillType::ANY as i32 {
            return Some(SkillType::to_enum(skill_id));
        }
        return None;
    }
}

impl ConstructionRecipeDesc {
    pub fn get_skill_type(&self) -> Option<SkillType> {
        let skill_id = match self.experience_per_progress.first() {
            Some(exp) => exp.skill_id,
            None => 0,
        };
        if skill_id > SkillType::ANY as i32 {
            return Some(SkillType::to_enum(skill_id));
        }
        return None;
    }
}

impl DeconstructionRecipeDesc {
    pub fn get_skill_type(&self) -> Option<SkillType> {
        let skill_id = match self.experience_per_progress.first() {
            Some(exp) => exp.skill_id,
            None => 0,
        };
        if skill_id > SkillType::ANY as i32 {
            return Some(SkillType::to_enum(skill_id));
        }
        return None;
    }
}

impl ResourcePlacementRecipeDesc {
    pub fn get_skill_type(&self) -> Option<SkillType> {
        let skill_id = match self.experience_per_progress.first() {
            Some(exp) => exp.skill_id,
            None => 0,
        };
        if skill_id > SkillType::ANY as i32 {
            return Some(SkillType::to_enum(skill_id));
        }
        return None;
    }
}

impl EmpireNotificationType {
    pub fn from_i32(value: i32) -> EmpireNotificationType {
        match value {
            1 => EmpireNotificationType::NewMember,
            2 => EmpireNotificationType::MarkedForSiege,
            3 => EmpireNotificationType::StartedSiege,
            4 => EmpireNotificationType::StartedDefense,
            5 => EmpireNotificationType::SuccessfulSiege,
            6 => EmpireNotificationType::SuccessfulDefense,
            7 => EmpireNotificationType::FailedSiege,
            8 => EmpireNotificationType::FailedDefense,
            9 => EmpireNotificationType::MemberLeft,
            10 => EmpireNotificationType::WatchtowerBuilt,
            11 => EmpireNotificationType::ClaimJoined,
            12 => EmpireNotificationType::ClaimLeft,
            13 => EmpireNotificationType::Donation,
            14 => EmpireNotificationType::DonationByProxy,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

impl RegionSignInParameters {
    pub fn get(ctx: &ReducerContext) -> Option<RegionSignInParameters> {
        if let Some(globals) = ctx.db.globals().version().find(0) {
            return ctx.db.region_sign_in_parameters().region_id().find(globals.region_index);
        }

        None
    }
}
