use spacetimedb::{ReducerContext, Table};

use crate::{
    game::handlers::authentication::has_role,
    messages::{
        authentication::Role,
        generic::{gated_features, GatedFeature},
    },
};

fn normalize_feature_key(feature: String) -> Result<String, String> {
    let feature = feature.trim().to_string();
    if feature.is_empty() {
        return Err("Feature key cannot be empty".into());
    }

    Ok(feature)
}

#[spacetimedb::reducer]
pub fn admin_gated_feature_add(ctx: &ReducerContext, feature: String) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let feature = normalize_feature_key(feature)?;
    if ctx.db.gated_features().feature().find(&feature).is_some() {
        return Ok(());
    }

    ctx.db.gated_features().insert(GatedFeature { feature });

    Ok(())
}

#[spacetimedb::reducer]
pub fn admin_gated_feature_remove(ctx: &ReducerContext, feature: String) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let feature = normalize_feature_key(feature)?;
    if ctx.db.gated_features().feature().find(&feature).is_some() {
        ctx.db.gated_features().feature().delete(&feature);
    }

    Ok(())
}
