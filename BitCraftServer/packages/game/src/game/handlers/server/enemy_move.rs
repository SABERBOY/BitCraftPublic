use crate::{
    game::{entities::location::MobileEntityState, game_state::unix_ms, handlers::authentication::has_role},
    messages::{action_request::EnemyMoveRequest, authentication::Role},
    mobile_entity_state,
};
use spacetimedb::ReducerContext;

#[spacetimedb::reducer]
pub fn enemy_move(ctx: &ReducerContext, request: EnemyMoveRequest) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Invalid permissions".into());
    }

    reduce(ctx, request);

    Ok(())
}

#[spacetimedb::reducer]
pub fn enemy_move_batch(ctx: &ReducerContext, requests: Vec<EnemyMoveRequest>) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Invalid permissions".into());
    }

    for request in requests {
        reduce(ctx, request);
    }

    Ok(())
}

fn reduce(ctx: &ReducerContext, request: EnemyMoveRequest) {
    let start_offset_coordinates = request.origin;
    let target_offset_coordinates = request.destination;

    // update location
    let mobile_entity = MobileEntityState {
        entity_id: request.entity_id,
        chunk_index: request.chunk_index,
        timestamp: unix_ms(ctx.timestamp),
        location_x: start_offset_coordinates.x,
        location_z: start_offset_coordinates.z,
        destination_x: target_offset_coordinates.x,
        destination_z: target_offset_coordinates.z,
        dimension: target_offset_coordinates.dimension,
        is_walking: false,
        _pad1: 0,
        _pad2: 0,
        _pad3: 0,
    };
    if ctx.db.mobile_entity_state().entity_id().find(request.entity_id).is_some() {
        ctx.db.mobile_entity_state().entity_id().update(mobile_entity);
    }
}
