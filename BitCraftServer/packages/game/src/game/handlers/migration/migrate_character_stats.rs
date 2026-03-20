use spacetimedb::{log, ReducerContext, Table};

use crate::{
    game::handlers::authentication::has_role,
    messages::{
        authentication::Role,
        components::character_stats_state,
        static_data::{character_stat_desc, CharacterStatType},
    },
};

#[spacetimedb::reducer]
pub fn migrate_character_stats(ctx: &ReducerContext) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let types: Vec<i32> = ctx.db.character_stat_desc().iter().map(|cs| cs.stat_type).collect();
    let num_types = types.len();

    let mut count = 0;
    for mut stats in ctx.db.character_stats_state().iter() {
        let previous_stats_count = stats.values.len();
        if previous_stats_count < num_types {
            stats.values.extend(vec![0.0; num_types - previous_stats_count]);
            for t in previous_stats_count..num_types {
                let value = ctx.db.character_stat_desc().stat_type().find(t as i32).unwrap().value;
                stats.set(CharacterStatType::to_enum(t as i32), value);
            }
        }
        count += 1;
        ctx.db.character_stats_state().entity_id().update(stats);
    }

    log::info!("Edited {count} character stats");

    Ok(())
}
