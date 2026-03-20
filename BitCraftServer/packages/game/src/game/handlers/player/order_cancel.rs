use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

use crate::{
    game::game_state,
    messages::{
        action_request::PlayerOrderCancelRequest,
        components::{building_state, buy_order_state, mobile_entity_state, sell_order_state, HealthState},
    },
    unwrap_or_err,
};

#[spacetimedb::reducer]
#[feature_gate("trade")]
pub fn order_cancel(ctx: &ReducerContext, request: PlayerOrderCancelRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;
    HealthState::check_incapacitated(ctx, actor_id, true)?;

    let building = unwrap_or_err!(
        ctx.db.building_state().entity_id().find(request.building_entity_id),
        "Building does not exist"
    );

    let coordinates = ctx.db.mobile_entity_state().entity_id().find(actor_id).unwrap().coordinates();
    if building.distance_to(ctx, &coordinates) > 5 {
        return Err("Too far".into());
    }

    let claim_entity_id = building.claim_entity_id;

    // Find all sell_orders matching the price and item id, and collect items from those, sorted by increasing price (with timestamp for tie-breaking)
    if let Some(order) = ctx.db.sell_order_state().entity_id().find(request.auction_listing_entity_id) {
        if order.owner_entity_id != actor_id {
            return Err("You are not the owner of this listing".into());
        }
        if order.claim_entity_id != claim_entity_id {
            return Err("You cannot cancel this listing from there".into());
        }

        // Refund "sold" items
        order.cancel_sell_order(ctx);
    } else if let Some(order) = ctx.db.buy_order_state().entity_id().find(request.auction_listing_entity_id) {
        if order.owner_entity_id != actor_id {
            return Err("You are not the owner of this listing".into());
        }
        if order.claim_entity_id != claim_entity_id {
            return Err("You cannot cancel this listing from there".into());
        }
        // Refund coins
        order.cancel_buy_order(ctx);
    }

    Ok(())
}
