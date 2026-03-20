use bitcraft_macro::feature_gate;
use spacetimedb::ReducerContext;

#[spacetimedb::reducer]
#[feature_gate]
pub fn synchronize_time(_ctx: &ReducerContext, _client_time: f64) -> Result<(), String> {
    Ok(())
}
