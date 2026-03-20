use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

use crate::game::entities;
use crate::game::game_state::{self, game_state_filters};
use crate::inter_module::send_inter_module_message;
use crate::messages::components::{building_state, InventoryState, Permission, PermissionState};
use crate::messages::empire_shared::empire_chunk_state;
use crate::messages::game_util::{ItemStack, ItemType};
use crate::messages::inter_module::EmpireAddCurrencyMsg;
use crate::messages::static_data::{building_buff_desc, building_desc, item_desc};
use crate::{unwrap_or_err, PlayerTimestampState};

#[spacetimedb::reducer]
#[feature_gate]
pub fn activate_building_buff(ctx: &ReducerContext, building_entity_id: u64) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;
    PlayerTimestampState::refresh(ctx, actor_id, ctx.timestamp);

    let player_coord = game_state_filters::coordinates_float(ctx, actor_id);

    let building = unwrap_or_err!(ctx.db.building_state().entity_id().find(building_entity_id), "Invalid building");

    /*
    // this code checks building alignment based on empire, we want territory alignment
    let empire_entity_id = EmpireState::get_building_empire_entity_id(ctx, building_entity_id);
    if empire_entity_id == 0 {
        return Err("Only empire aligned buildings can provide buffs".into());
    }
    */

    let building_location = game_state_filters::coordinates_any(ctx, building_entity_id);
    let chunk_index = building_location.chunk_coordinates().chunk_index();
    let empire_chunk = unwrap_or_err!(
        ctx.db.empire_chunk_state().chunk_index().find(chunk_index),
        "Only empire aligned buildings can provide buffs"
    );

    let empire_entity_id = empire_chunk.empire_entity_id;

    let building_buff = unwrap_or_err!(
        ctx.db
            .building_buff_desc()
            .building_id()
            .filter(building.building_description_id)
            .next(),
        "This building can't provide buffs"
    );

    if !PermissionState::can_interact_with_building(ctx, actor_id, &building, Permission::Usage) {
        return Err("You don't have the permission to use this building".into());
    }

    // if player is not inside a building check if the request is for
    // an unenterable building (e.g. firepit) and if they are close enough
    let building_desc = unwrap_or_err!(
        ctx.db.building_desc().id().find(&building.building_description_id),
        "Unknown building type"
    );

    if building_desc.unenterable {
        // Temporary: allow a distance of 2 for when you right-click on building while moving and end up 1 tile too far by completing the current move
        if building.distance_to(ctx, &player_coord.into()) > 2 {
            return Err("Too far".into());
        }
    } else {
        let player_building =
            game_state_filters::building_at_coordinates(ctx, &game_state_filters::coordinates_float(ctx, actor_id).into());
        match player_building {
            Some(player_building) => {
                if player_building.entity_id != building_entity_id {
                    return Err("Player is inside another building".into());
                }
            }
            None => return Err("Player isn't inside a building".into()),
        }
    }

    if building_buff.empire_currency_cost > 0 {
        let empire_currency_id = ctx.db.item_desc().tag().filter("Empire Currency").next().unwrap().id;
        let cost = vec![ItemStack::new(
            ctx,
            empire_currency_id,
            ItemType::Item,
            building_buff.empire_currency_cost,
        )];
        InventoryState::withdraw_from_player_inventory_and_nearby_deployables(ctx, actor_id, &cost, |x| building.distance_to(ctx, &x))?;

        send_inter_module_message(
            ctx,
            crate::messages::inter_module::MessageContentsV2::EmpireAddCurrency(EmpireAddCurrencyMsg {
                empire_entity_id,
                amount: building_buff.empire_currency_cost as u32,
            }),
            crate::inter_module::InterModuleDestination::Global,
        );
    }

    // gain potential buffs
    for buff_effect in &building_buff.buffs {
        entities::buff::activate(ctx, actor_id, buff_effect.buff_id, buff_effect.duration, None)?;
    }

    Ok(())
}
