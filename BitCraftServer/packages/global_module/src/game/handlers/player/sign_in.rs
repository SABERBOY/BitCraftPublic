use bitcraft_macro::feature_gate;
use spacetimedb::{log, ReducerContext, Table};

use crate::{
    game::{
        game_state,
        handlers::authentication::{has_role, is_authenticated},
    },
    messages::{
        action_request::PlayerSignInRequest,
        authentication::{blocked_identity, developer, Role},
        components::*,
        empire_schema::*,
        global::player_shard_state,
    },
};

#[spacetimedb::reducer]
#[feature_gate]
pub fn sign_in(ctx: &ReducerContext, _request: PlayerSignInRequest) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, false)?;

    let _ = ctx
        .db
        .signed_in_player_state()
        .try_insert(SignedInPlayerState { entity_id: actor_id });

    // Update last viewed empire messages
    if let Some(mut player_log) = ctx.db.empire_player_log_state().entity_id().find(&actor_id) {
        if let Some(empire_log) = ctx.db.empire_log_state().entity_id().find(&player_log.empire_entity_id) {
            player_log.last_viewed = empire_log.last_posted;
            ctx.db.empire_player_log_state().entity_id().update(player_log);
        }
    }

    // Claim pending shards
    if let Some(unclaimed_shards) = ctx.db.unclaimed_shards_state().identity().find(&ctx.sender) {
        let mut vault = ctx.db.player_shard_state().entity_id().find(&actor_id).unwrap();
        vault.shards += unclaimed_shards.shards;
        ctx.db.player_shard_state().entity_id().update(vault);
        ctx.db.unclaimed_shards_state().identity().delete(&ctx.sender);
    }

    Ok(())
}

// Called everytime a new client connects
#[spacetimedb::reducer(client_connected)]
#[feature_gate]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    if let Some(developer) = ctx.db.developer().identity().find(ctx.sender) {
        log::info!(
            "Developer identity connected for developer: {}, service: {}",
            developer.developer_name,
            developer.service_name
        );
        return Ok(());
    }

    if has_role(ctx, &ctx.sender, Role::SkipQueue) {
        return Ok(());
    }

    if ctx.db.blocked_identity().identity().find(ctx.sender).is_some() || !is_authenticated(ctx, &ctx.sender) {
        log::info!("Blocking identity {}", ctx.sender.to_hex());
        return Err("Unauthorized".into());
    }

    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user_state().identity().find(&ctx.sender) {
        ctx.db.signed_in_player_state().entity_id().delete(user.entity_id);
    }
}
