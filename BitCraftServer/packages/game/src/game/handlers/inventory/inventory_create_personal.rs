use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

use crate::{
    building_desc, building_state, game::game_state, inventory_state, mobile_entity_state, unwrap_or_err, BuildingCategory, HealthState,
    InventoryState, PlayerTimestampState,
};

use super::inventory_helper;

#[spacetimedb::reducer]
#[feature_gate]
pub fn inventory_create_personal(ctx: &ReducerContext, building_entity_id: u64) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;

    HealthState::check_incapacitated(ctx, actor_id, true)?;

    PlayerTimestampState::refresh(ctx, actor_id, ctx.timestamp);

    let building_state = unwrap_or_err!(ctx.db.building_state().entity_id().find(&building_entity_id), "Invalid building");
    let building_desc = unwrap_or_err!(
        ctx.db.building_desc().id().find(&building_state.building_description_id),
        "Invalid building description"
    );
    let bank_function = unwrap_or_err!(
        building_desc.get_function(ctx, BuildingCategory::Bank),
        "Building doesn't allow personal inventories."
    );

    let player_location = unwrap_or_err!(ctx.db.mobile_entity_state().entity_id().find(&actor_id), "Invalid player location").coordinates();
    inventory_helper::validate_interact(ctx, actor_id, player_location, building_entity_id, 0)?;

    if ctx
        .db
        .inventory_state()
        .entity_id()
        .find(&building_entity_id)
        .iter()
        .any(|x| x.player_owner_entity_id == actor_id)
    {
        return Err("Personal inventory already exists".into());
    }

    if !InventoryState::new(
        ctx,
        bank_function.storage_slots + bank_function.cargo_slots,
        bank_function.item_slot_size,
        bank_function.cargo_slot_size,
        bank_function.storage_slots,
        building_entity_id,
        actor_id,
        None,
    ) {
        return Err("Failed to insert InventoryState".into());
    }

    Ok(())
}
