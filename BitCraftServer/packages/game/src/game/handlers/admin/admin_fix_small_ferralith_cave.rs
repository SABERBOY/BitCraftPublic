use spacetimedb::{log, ReducerContext, Table};

use crate::{
    game::handlers::authentication::has_role,
    messages::{
        authentication::Role,
        components::{building_state, dimension_description_state, location_state},
    },
};

#[spacetimedb::reducer]
pub fn admin_flip_interior_instance_doors(ctx: &ReducerContext, interior_instance_id: i32) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let mut count = 0;

    for dimension in ctx
        .db
        .dimension_description_state()
        .iter()
        .filter(|d| d.interior_instance_id == interior_instance_id)
    {
        for location in ctx.db.location_state().dimension_filter(dimension.dimension_id) {
            if let Some(mut building) = ctx.db.building_state().entity_id().find(location.entity_id) {
                if building.building_description_id == 1593993049 {
                    // 1593993049 is Building_Exit
                    if building.direction_index >= 6 {
                        building.direction_index -= 6;
                    } else {
                        building.direction_index += 6;
                    }
                    count += 1;
                    ctx.db.building_state().entity_id().update(building);
                }
            }
        }
    }

    log::info!("{count} cave entrances updated");
    Ok(())
}
