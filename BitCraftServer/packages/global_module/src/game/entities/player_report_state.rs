use spacetimedb::{ReducerContext, Table};

use crate::{
    game::game_state,
    messages::components::{player_report_state, player_report_state_timestamp, PlayerReportState, PlayerReportStateTimestamp},
};

impl PlayerReportState {
    pub fn inter_module_insert(ctx: &ReducerContext, report_state: PlayerReportState) -> Result<(), String> {
        ctx.db.player_report_state_timestamp().try_insert(PlayerReportStateTimestamp {
            entity_id: report_state.entity_id,
            timestamp: game_state::unix(ctx.timestamp),
        })?;
        ctx.db.player_report_state().try_insert(report_state)?;
        Ok(())
    }
}
