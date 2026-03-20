use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

use crate::{
    game::{
        discovery::Discovery,
        game_state::{self, game_state_filters},
        reducer_helpers::player_action_helpers,
    },
    inter_module::*,
    messages::{components::*, empire_shared::*, game_util::ItemStack, inter_module::*},
    unwrap_or_return,
};

#[spacetimedb::reducer]
#[feature_gate("empire")]
pub fn empire_collect_hexite_capsule(ctx: &ReducerContext, request: EmpireCollectHexiteCapsuleRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;

    HealthState::check_incapacitated(ctx, actor_id, true)?;

    let player_location = game_state_filters::coordinates_any(ctx, actor_id);
    let foundry_location = game_state_filters::coordinates(ctx, request.building_entity_id);
    if player_location.distance_to(foundry_location) > 3 {
        return Err("Too far".into());
    }

    if !EmpirePlayerDataState::has_permission_to_use_empire_building(
        ctx,
        actor_id,
        request.building_entity_id,
        EmpirePermission::CollectHexiteCapsule,
    ) {
        return Err("You don't have the permissions to collect a hexite capsule".into());
    }

    let carried_cargo_id = InventoryState::get_player_cargo_id(ctx, actor_id);
    if carried_cargo_id != 0 {
        return Err("Already carrying a cargo".into());
    }

    send_inter_module_message(
        ctx,
        crate::messages::inter_module::MessageContentsV2::EmpireCollectHexiteCapsule(EmpireCollectHexiteCapsuleMsg {
            building_entity_id: request.building_entity_id,
            player_entity_id: actor_id,
        }),
        crate::inter_module::InterModuleDestination::Global,
    );

    Ok(())
}

pub fn handle_destination_result_on_sender(ctx: &ReducerContext, request: EmpireCollectHexiteCapsuleMsg, error: Option<String>) {
    if error.is_none() {
        //Create cargo only if reducer succeeds

        let mut player_inventory = unwrap_or_return!(
            InventoryState::get_player_inventory(ctx, request.player_entity_id),
            "Player has no inventory"
        );
        let item_stack = vec![ItemStack::hexite_capsule()];

        player_inventory.add_multiple_with_overflow(ctx, &item_stack);
        ctx.db.inventory_state().entity_id().update(player_inventory);

        let mut discovery = Discovery::new(request.player_entity_id);
        discovery.acquire_item_stack(ctx, &item_stack[0]);
        discovery.commit(ctx);

        player_action_helpers::post_reducer_update_cargo(ctx, request.player_entity_id);
    } else {
        PlayerNotificationEvent::new_event(ctx, request.player_entity_id, error.unwrap(), NotificationSeverity::ReducerError);
    }
}
