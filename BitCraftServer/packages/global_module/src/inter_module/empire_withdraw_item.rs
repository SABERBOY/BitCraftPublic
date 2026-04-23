use spacetimedb::ReducerContext;

use crate::{
    messages::{empire_shared::*, inter_module::*, static_data::item_desc},
    unwrap_or_err,
};

pub fn process_message_on_destination(ctx: &ReducerContext, request: EmpireWithdrawItemMsg) -> Result<(), String> {
    let player_empire_data = unwrap_or_err!(
        ctx.db.empire_player_data_state().entity_id().find(request.player_entity_id),
        "Player must be part of an empire to withdraw"
    );

    let mut empire = unwrap_or_err!(
        ctx.db.empire_state().entity_id().find(player_empire_data.empire_entity_id),
        "This empire does not exist"
    );

    let _player_data = unwrap_or_err!(
        ctx.db.empire_player_data_state().entity_id().find(request.player_entity_id),
        "You are not part of an empire"
    );

    if request.is_cargo {
        // for now no cargo can be withdrawd
        return Err("This cargo can't be withdrawn".into());
    } else {
        let item_desc = unwrap_or_err!(ctx.db.item_desc().id().find(request.item_id), "Unknown withdrawn item");
        match item_desc.tag.as_str() {
            "Empire Currency" => {
                if empire.empire_currency_treasury < request.count {
                    return Err("Not enough in the empire treasury".into());
                }
                empire.empire_currency_treasury -= request.count;
            }
            _ => return Err("This item can't be withdrawn".into()),
        }
    }

    EmpireState::update_shared(ctx, empire, super::InterModuleDestination::AllOtherRegions);

    Ok(())
}
