use bitcraft_macro::feature_gate;
use spacetimedb::{log, ReducerContext};

use crate::game::reducer_helpers::player_action_helpers;
use crate::{
    building_repairs_desc,
    game::game_state,
    messages::{action_request::ClaimPurchaseSuppliesFromPlayerRequest, components::*},
    unwrap_or_err,
};

#[spacetimedb::reducer]
#[feature_gate]
pub fn claim_purchase_supplies_from_player(ctx: &ReducerContext, request: ClaimPurchaseSuppliesFromPlayerRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;
    PlayerTimestampState::refresh(ctx, actor_id, ctx.timestamp);

    let building_entity_id = request.building_entity_id;
    let building = unwrap_or_err!(ctx.db.building_state().entity_id().find(&building_entity_id), "No such building.");

    let claim = unwrap_or_err!(ctx.db.claim_state().entity_id().find(&building.claim_entity_id), "No such claim.");
    let mut claim_local = claim.local_state(ctx);

    let max_supplies = ctx
        .db
        .claim_tech_state()
        .entity_id()
        .find(claim.entity_id)
        .unwrap()
        .max_supplies(ctx) as i32;

    let supplies_threshold = claim_local.supplies_purchase_threshold as i32;
    if claim_local.supplies >= supplies_threshold {
        return Err("Claim is not purchasing supplies at this point".into());
    }

    // find stack with a repair kit
    let mut inventory = unwrap_or_err!(InventoryState::get_player_inventory(ctx, actor_id), "Player has no inventory");
    if let Some(cargo_item_stack) = inventory.remove_at(inventory.cargo_index as usize) {
        let repair_value = match ctx.db.building_repairs_desc().cargo_id().find(&cargo_item_stack.item_id) {
            Some(rep) => rep.repair_value,
            None => return Err("Claims can only be charged with supplies.".into()),
        }
        .min(max_supplies - claim_local.supplies);

        ctx.db.inventory_state().entity_id().update(inventory);

        let paid_repairs = repair_value.min(supplies_threshold - claim_local.supplies);

        if paid_repairs < request.paid_supplies {
            return Err("The claim is no longer purchasing that many supplies.".into());
        }

        if claim_local.supplies_purchase_price != request.price_per_supply {
            return Err("The claim updated its claim purchase policies, please try again.".into());
        }

        let amount = f32::ceil(paid_repairs as f32 * claim_local.supplies_purchase_price) as i32;
        let amount = i32::min(amount, claim_local.treasury as i32);

        log::info!(
            "Paying for {paid_repairs} supplies at {} each, treasury is {}, final amount = {amount}",
            claim_local.supplies_purchase_price,
            claim_local.treasury
        );
        claim_local.treasury -= amount as u32;
        let _ = claim_local.update_supplies_and_commit(ctx, repair_value as f32, false);

        if !InventoryState::add_to_player_wallet_and_commit(ctx, actor_id, amount, 0) {
            return Err("You don't have enough room to collect the payment.".into());
        }

        player_action_helpers::post_reducer_update_cargo(ctx, actor_id);
    } else {
        return Err("You don't carry claim supplies".into());
    }
    Ok(())
}
