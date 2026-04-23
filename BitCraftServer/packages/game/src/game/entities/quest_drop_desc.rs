use spacetimedb::ReducerContext;

use crate::messages::{components::quest_chain_state, game_util::ItemStack, static_data::{QuestDropDesc, quest_drop_desc}};

impl QuestDropDesc {
    pub fn roll_extraction(
        ctx: &ReducerContext,
        player_entity_id: u64,
        extraction_recipe_id: i32,
        damage_output: i32,
    ) -> Option<Vec<ItemStack>> {
        if extraction_recipe_id == 0 {
            return None;
        }

        let output: Vec<ItemStack> = ctx
            .db
            .quest_drop_desc()
            .extraction_id()
            .filter(extraction_recipe_id)
            .filter_map(|drop| drop.roll(ctx, player_entity_id, damage_output))
            .collect();
    
        (!output.is_empty()).then_some(output)
    }

    pub fn roll_enemy(
        ctx: &ReducerContext,
        player_entity_id: u64,
        enemy_type_id: i32
    ) -> Option<Vec<ItemStack>> {
        if enemy_type_id == 0 {
            return None;
        }

        let output: Vec<ItemStack> = ctx
            .db
            .quest_drop_desc()
            .enemy_id()
            .filter(enemy_type_id)
            .filter_map(|drop| drop.roll(ctx, player_entity_id, 1))
            .collect();
    
        (!output.is_empty()).then_some(output)
    }

    pub fn roll_item_list(
        ctx: &ReducerContext,
        player_entity_id: u64,
        item_list_id: i32,
        num_rolls: i32
    ) -> Option<Vec<ItemStack>> {
        if item_list_id == 0 {
            return None;
        }

        let output: Vec<ItemStack> = ctx
            .db
            .quest_drop_desc()
            .item_list_id()
            .filter(item_list_id)
            .filter_map(|drop| drop.roll(ctx, player_entity_id, num_rolls))
            .collect();
    
        (!output.is_empty()).then_some(output)
    }
    
    pub fn roll(
        &self,
        ctx: &ReducerContext,
        player_entity_id: u64,
        num_rolls: i32,
    ) -> Option<ItemStack> {
        ctx.db
            .quest_chain_state()
            .player_entity_id()
            .filter(player_entity_id)
            .find(|qcs| qcs.quest_chain_desc_id == self.required_quest_id)
            .and_then(|qcs| {
                let correct_stage = self.required_stage_id == 0 || self.required_stage_id == qcs.stage_id;
                if correct_stage && !qcs.completed {
                    self.item_drop.roll(ctx, num_rolls)
                } else {
                    None
                }
            })
    }
}