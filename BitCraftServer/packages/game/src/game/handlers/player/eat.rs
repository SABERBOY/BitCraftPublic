use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

use crate::{
    game::{
        entities::{self, building_state::InventoryState},
        game_state::{self},
    },
    messages::{action_request::PlayerEatRequest, components::*, static_data::*},
    unwrap_or_err,
};

#[spacetimedb::reducer]
#[feature_gate]
pub fn eat(ctx: &ReducerContext, request: PlayerEatRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;

    HealthState::check_incapacitated(ctx, actor_id, true)?;

    PlayerTimestampState::refresh(ctx, actor_id, ctx.timestamp);

    let is_sleeping = PlayerActionState::is_player_doing_action(ctx, &actor_id, &PlayerActionType::Sleep)?;
    if is_sleeping {
        return Err("You cannot eat while you sleep.".into());
    }

    let mut inventory = unwrap_or_err!(InventoryState::get_player_inventory(ctx, actor_id), "Player has no inventory");
    let contents = unwrap_or_err!(inventory.get_pocket_contents(request.pocket_index as usize), "Invalid pocket");
    if contents.item_id <= 0 {
        return Err("Nothing to eat".into());
    }

    let food = unwrap_or_err!(ctx.db.food_desc().item_id().find(&contents.item_id), "Item can't be eaten");

    if !food.consumable_while_in_combat && ThreatState::in_combat(ctx, actor_id) {
        return Err("Cannot eat this while in combat".into());
    }

    let character_stats_state = unwrap_or_err!(ctx.db.character_stats_state().entity_id().find(&actor_id), "Player has no stats");

    if food.hp != 0f32 {
        apply_health(ctx, actor_id, &food, &character_stats_state)?;
    }

    if food.stamina != 0f32 {
        apply_stamina(ctx, actor_id, &food, &character_stats_state)?;
    }

    if food.hunger != 0f32 {
        SatiationState::add_player_satiation(ctx, actor_id, food.hunger);
    }

    if food.teleportation_energy != 0f32 {
        let mut teleportation_energy_state = TeleportationEnergyState::get(ctx, actor_id);
        if teleportation_energy_state.add_energy(ctx, food.teleportation_energy) {
            teleportation_energy_state.update(ctx);
        }
    }

    // gain potential buffs
    for buff_effect in &food.buffs {
        entities::buff::activate(ctx, actor_id, buff_effect.buff_id, buff_effect.duration, None)?;
    }

    inventory.remove_quantity_at(request.pocket_index as usize, 1);
    ctx.db.inventory_state().entity_id().update(inventory);

    Ok(())
}

fn apply_health(ctx: &ReducerContext, actor_id: u64, food: &FoodDesc, character_stats_state: &CharacterStatsState) -> Result<(), String> {
    let mut health_state = unwrap_or_err!(ctx.db.health_state().entity_id().find(&actor_id), "Player has no health state");

    //Gain or lose health in the full range
    if food.up_to_hp == 0f32 {
        if health_state.add_health_delta_clamped(
            food.hp,
            0f32,
            character_stats_state.get(CharacterStatType::MaxHealth),
            ctx.timestamp,
        ) {
            ctx.db.health_state().entity_id().update(health_state);
        }
    }
    //Gain health up to static-data value
    else if food.hp > 0f32 {
        if health_state.add_health_delta_clamped(food.hp, 0f32, food.up_to_hp, ctx.timestamp) {
            ctx.db.health_state().entity_id().update(health_state);
        }
    }
    //Lose health up to static-data value
    else if health_state.add_health_delta_clamped(
        food.hp,
        food.up_to_hp,
        character_stats_state.get(CharacterStatType::MaxHealth),
        ctx.timestamp,
    ) {
        ctx.db.health_state().entity_id().update(health_state);
    }

    Ok(())
}

fn apply_stamina(ctx: &ReducerContext, actor_id: u64, food: &FoodDesc, character_stats_state: &CharacterStatsState) -> Result<(), String> {
    let mut stamina_state = unwrap_or_err!(ctx.db.stamina_state().entity_id().find(&actor_id), "Player has no stamina state");

    //Gain or lose stamina in the full range
    if food.up_to_stamina == 0f32 {
        if stamina_state.add_stamina_delta_clamped(
            food.stamina,
            0f32,
            character_stats_state.get(CharacterStatType::MaxStamina),
            ctx.timestamp,
        ) {
            ctx.db.stamina_state().entity_id().update(stamina_state);
        }
    }
    //Gain stamina up to static-data value
    else if food.stamina > 0f32 {
        if stamina_state.add_stamina_delta_clamped(food.stamina, 0f32, food.up_to_stamina, ctx.timestamp) {
            ctx.db.stamina_state().entity_id().update(stamina_state);
        }
    }
    //Lose stamina up to static-data value
    else if stamina_state.add_stamina_delta_clamped(
        food.stamina,
        food.up_to_stamina,
        character_stats_state.get(CharacterStatType::MaxStamina),
        ctx.timestamp,
    ) {
        ctx.db.stamina_state().entity_id().update(stamina_state);
    }

    Ok(())
}
