use bitcraft_macro::shared_table_reducer;
use spacetimedb::{ReducerContext, Table};

use crate::game::game_state::{create_entity, unix};
use crate::messages::action_request::CreatePlayerReportRequest;
use crate::messages::components::{
    chat_message_state, player_report_state, player_username_state, ChatChannel, ChatMessageState, PlayerReportState,
};
use crate::{game::handlers::authentication::has_role, messages::authentication::Role, unwrap_or_err};

const CONTEXT_SIZE: usize = 5;

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_create_chat_message(
    ctx: &ReducerContext,
    channel_id: ChatChannel,
    username: String,
    title_id: i32,
    target_id: u64,
    new_message_text: String,
) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        return Err("Unauthorized".into());
    }

    let message_entity_id = create_entity(ctx);
    if ctx
        .db
        .chat_message_state()
        .try_insert(ChatMessageState {
            entity_id: message_entity_id,
            username,
            title_id,
            text: new_message_text,
            timestamp: unix(ctx.timestamp),
            owner_entity_id: 0,
            target_id,
            channel_id: channel_id as i32,
            //language_code: None //I18N
        })
        .is_err()
    {
        return Err("Failed to insert chat message".into());
    }

    Ok(())
}

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_modify_chat_message(ctx: &ReducerContext, entity_id: u64, new_message_text: String) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        return Err("Unauthorized".into());
    }

    let mut chat_message = unwrap_or_err!(
        ctx.db.chat_message_state().entity_id().find(entity_id),
        "Chat message does not exist"
    );

    chat_message.text = new_message_text;
    ctx.db.chat_message_state().entity_id().update(chat_message);

    Ok(())
}

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_delete_chat_message(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Gm) {
        return Err("Unauthorized".into());
    }

    let chat_message = unwrap_or_err!(
        ctx.db.chat_message_state().entity_id().find(entity_id),
        "Chat message does not exist"
    );

    ctx.db.chat_message_state().delete(chat_message);

    Ok(())
}

#[shared_table_reducer]
#[spacetimedb::reducer]
pub fn admin_create_entity_name_report(
    ctx: &ReducerContext,
    report_type: String,
    entity_id: u64,
    entity_name: String,
    message: String,
) -> Result<(), String> {
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    let report_entity_id = create_entity(ctx);
    ctx.db.player_report_state().insert(PlayerReportState {
        entity_id: report_entity_id,
        reporter_entity_id: 0, // Auto-mod
        reported_player_entity_id: entity_id,
        reported_player_username: entity_name,
        report_type,
        report_message: message,
        reported_chat_message: None,
        chat_channel_context: None,
        chat_user_context: None,
        actioned: false,
    });

    Ok(())
}

pub fn build_player_report(ctx: &ReducerContext, request: CreatePlayerReportRequest) -> Result<PlayerReportState, String> {
    // auth
    if !has_role(ctx, &ctx.sender, Role::Admin) {
        return Err("Unauthorized".into());
    }

    // optional chat
    let chat_opt = ctx.db.chat_message_state().entity_id().find(request.chat_message_id);

    // username (fallback)
    let username = ctx
        .db
        .player_username_state()
        .entity_id()
        .find(request.reported_player_entity_id)
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Anonymous".to_string());

    // contexts
    let (reported_chat, channel_ctx, user_ctx) = if let Some(chat) = chat_opt.clone() {
        // channel context
        let channel_messages: Vec<ChatMessageState> = ctx.db.chat_message_state().channel_id().filter(chat.channel_id).collect();
        let idx = channel_messages.iter().position(|v| v.entity_id == chat.entity_id).unwrap_or(0);
        let start = idx.saturating_sub(CONTEXT_SIZE);
        let end = (idx + CONTEXT_SIZE).min(channel_messages.len());
        let channel_ctx = channel_messages[start..end].to_vec();

        // user context
        let user_messages: Vec<ChatMessageState> = ctx.db.chat_message_state().owner_entity_id().filter(chat.owner_entity_id).collect();
        let uidx = user_messages.iter().position(|v| v.entity_id == chat.entity_id).unwrap_or(0);
        let ustart = uidx.saturating_sub(CONTEXT_SIZE);
        let uend = (uidx + CONTEXT_SIZE).min(user_messages.len());
        let user_ctx = user_messages[ustart..uend].to_vec();

        (Some(chat), Some(channel_ctx), Some(user_ctx))
    } else {
        (None, None, None)
    };

    // construct the row; persist elsewhere
    let entity_id = create_entity(ctx);
    Ok(PlayerReportState {
        entity_id,
        reporter_entity_id: request.reporter_entity_id,
        reported_player_entity_id: request.reported_player_entity_id,
        reported_player_username: username,
        report_type: request.report_type,
        report_message: request.message,
        reported_chat_message: reported_chat,
        chat_channel_context: channel_ctx,
        chat_user_context: user_ctx,
        actioned: true,
    })
}
