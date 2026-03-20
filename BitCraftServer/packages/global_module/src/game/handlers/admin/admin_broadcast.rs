use spacetimedb::ReducerContext;

use crate::{
    game::handlers::authentication::has_role,
    inter_module::send_inter_module_message,
    messages::{authentication::Role, generic::admin_broadcast},
};

#[spacetimedb::reducer]
pub fn admin_broadcast_msg(ctx: &ReducerContext, region: u8, title: String, message: String) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }
    reduce(ctx, region, title, message, false);
    Ok(())
}

pub fn reduce(ctx: &ReducerContext, region: u8, title: String, message: String, sign_out: bool) {
    if region == 0 {
        let mut broadcast = ctx.db.admin_broadcast().version().find(&0).unwrap();
        broadcast.title = title;
        broadcast.message = message;
        broadcast.sign_out = sign_out;
        broadcast.timestamp = ctx.timestamp;
        ctx.db.admin_broadcast().version().update(broadcast);
    } else {
        send_inter_module_message(
            ctx,
            crate::messages::inter_module::MessageContentsV2::AdminBroadcastMessage(
                crate::messages::inter_module::AdminBroadcastMessageMsg { title, message, sign_out },
            ),
            crate::inter_module::InterModuleDestination::Region(region),
        );
    }
}
