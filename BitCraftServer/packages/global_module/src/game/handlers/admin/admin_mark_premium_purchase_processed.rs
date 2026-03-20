use spacetimedb::{Identity, ReducerContext, Table};

use crate::{
    game::handlers::authentication::has_role,
    messages::{
        authentication::Role,
        generic::HubItemType,
        global::{granted_hub_item_state, premium_purchase_state, GrantedHubItemState},
    },
    unwrap_or_err,
};

#[spacetimedb::reducer]
pub fn admin_mark_premium_purchase_processed(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let mut premium_purchase_state = unwrap_or_err!(
        ctx.db.premium_purchase_state().entity_id().find(entity_id),
        "Unknown Premium Purchase"
    );

    if premium_purchase_state.processed {
        return Err("Premium Purchase has already been processed".into());
    }

    if let Some(collectible_desc_ids) = &premium_purchase_state.collectible_desc_ids {
        for collectible_desc_id in collectible_desc_ids {
            increment_collectible_balance(
                ctx,
                premium_purchase_state.identity,
                *collectible_desc_id,
                premium_purchase_state.quantity,
            );
        }
    }

    decrement_shard_balance(ctx, premium_purchase_state.identity, premium_purchase_state.price);

    premium_purchase_state.processed = true;
    ctx.db.premium_purchase_state().entity_id().update(premium_purchase_state);

    Ok(())
}

fn increment_collectible_balance(ctx: &ReducerContext, identity: Identity, collectible_desc_id: i32, quantity: u32) {
    if let Some(mut existing) = ctx
        .db
        .granted_hub_item_state()
        .identity_and_item_id()
        .filter((identity, collectible_desc_id))
        .find(|x| x.item_type == HubItemType::Collectible)
    {
        existing.balance += quantity;
        ctx.db.granted_hub_item_state().entity_id().update(existing);
        return;
    }

    ctx.db.granted_hub_item_state().insert(GrantedHubItemState {
        entity_id: 0,
        identity,
        item_type: HubItemType::Collectible,
        item_id: collectible_desc_id,
        balance: quantity,
    });
}

fn decrement_shard_balance(ctx: &ReducerContext, identity: Identity, price: u32) {
    let Some(mut existing) = ctx
        .db
        .granted_hub_item_state()
        .identity_and_item_id()
        .filter((identity, 0))
        .find(|x| x.item_type == HubItemType::HexiteShards)
    else {
        return;
    };

    existing.balance = existing.balance.saturating_sub(price);
    ctx.db.granted_hub_item_state().entity_id().update(existing);
}
