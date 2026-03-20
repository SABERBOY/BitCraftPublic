use bitcraft_macro::shared_table_reducer;
use spacetimedb::{log, ReducerContext, Table};

use crate::{
    game::handlers::authentication::has_role,
    messages::{
        authentication::Role,
        empire_shared::{empire_rank_state, EmpirePermission, EmpireRankState},
        static_data::empire_rank_desc,
    },
};
use strum::IntoEnumIterator;

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_update_empire_ranks(ctx: &ReducerContext) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        return Err("Unauthorized".into());
    }

    let num_values = EmpirePermission::iter().len();
    let mut count = 0;

    for mut rank in ctx.db.empire_rank_state().iter() {
        if rank.permissions.len() < num_values {
            let default_rank = ctx.db.empire_rank_desc().rank().find(rank.rank as i32).unwrap();
            rank.permissions
                .extend_from_slice(&default_rank.permissions[rank.permissions.len()..]);
            count += 1;
            EmpireRankState::update_shared(ctx, rank, crate::inter_module::InterModuleDestination::AllOtherRegions);
        }
    }

    log::info!("Updated {count} empire_rank_rate entries");
    Ok(())
}

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_push_empire_ranks_to_regions(ctx: &ReducerContext) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        return Err("Unauthorized".into());
    }

    let mut count = 0;
    for rank in ctx.db.empire_rank_state().iter() {
        count += 1;
        EmpireRankState::update_shared(ctx, rank, crate::inter_module::InterModuleDestination::AllOtherRegions);
    }

    log::info!("Shared update {count} empire_rank_rate entries");
    Ok(())
}
