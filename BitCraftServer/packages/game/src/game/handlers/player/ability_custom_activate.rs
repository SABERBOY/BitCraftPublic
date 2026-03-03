use std::time::Duration;

use spacetimedb::ReducerContext;

use crate::game::game_state::{self, game_state_filters};
use crate::game::reducer_helpers::player_action_helpers;
use crate::messages::components::{
    active_buff_state, enemy_state, health_state, player_state, stamina_state, AbilityState, AbilityType, AbilityTypeEnum,
    PlayerActionState, PlayerActionType, PlayerTimestampState, ThreatState,
};
use crate::messages::static_data::{ability_custom_desc, AbilityCustomDesc, AbilityUnlockDesc};
use crate::unwrap_or_err;

#[spacetimedb::reducer]
pub fn ability_custom_activate_start(
    ctx: &ReducerContext,
    ability_custom_id: i32,
    target_entity_id: u64,
    timestamp: u64,
) -> Result<(), String> {
    let custom_effect = unwrap_or_err!(ctx.db.ability_custom_desc().id().find(ability_custom_id), "Unknown Custom Effect");

    if custom_effect.cast_time == 0.0 {
        return ability_custom_activate(ctx, ability_custom_id, target_entity_id, timestamp);
    }

    let actor_id = game_state::actor_id(&ctx, true)?;
    PlayerTimestampState::refresh(ctx, actor_id, ctx.timestamp);

    let delay = Duration::from_secs_f32(custom_effect.cast_time);

    player_action_helpers::start_action(
        ctx,
        actor_id,
        PlayerActionType::AbilityCustom,
        Some(target_entity_id),
        None,
        delay,
        reduce(ctx, &custom_effect, target_entity_id, timestamp, true),
        timestamp,
    )
}

#[spacetimedb::reducer]
pub fn ability_custom_activate(ctx: &ReducerContext, ability_custom_id: i32, target_entity_id: u64, timestamp: u64) -> Result<(), String> {
    let custom_effect = unwrap_or_err!(ctx.db.ability_custom_desc().id().find(ability_custom_id), "Unknown Custom Effect");
    reduce(ctx, &custom_effect, target_entity_id, timestamp, false)
}

pub fn reduce(
    ctx: &ReducerContext,
    custom_effect: &AbilityCustomDesc,
    target_entity_id: u64,
    timestamp: u64,
    dry_run: bool,
) -> Result<(), String> {
    let actor_id = game_state::actor_id(&ctx, true)?;

    let ability_data = AbilityType::Custom(custom_effect.id);

    // Validate if ability is locked
    AbilityUnlockDesc::evaluate(ctx, actor_id, AbilityTypeEnum::Custom, ability_data)?;

    // Validate timestamps
    PlayerActionState::validate_timestamp_basic(ctx, actor_id, PlayerActionType::AbilityCustom, timestamp)?;
    if !dry_run && custom_effect.cast_time > 0.0 {
        // Make sure target and timestamp and action fit
        PlayerActionState::validate(ctx, actor_id, PlayerActionType::AbilityCustom, Some(target_entity_id))?;
        PlayerActionState::validate_action_timing(ctx, actor_id, PlayerActionType::AbilityCustom, timestamp)?;
    }

    // Check ability cooldown (only in dry-run, check is done as we set the cooldown in the real run)
    let ability_state = AbilityState::get(ctx, actor_id, ability_data);

    if dry_run && ability_state.is_under_cooldown(ctx, true) {
        return Err("Ability is under cooldown".into());
    }

    // Expend health
    if custom_effect.health_cost > 0 {
        let mut health = ctx.db.health_state().entity_id().find(actor_id).unwrap();
        if (health.health as i32) <= custom_effect.health_cost {
            return Err("Not enough health to trigger this ability".into());
        }
        if !dry_run {
            health.add_health_delta(-custom_effect.health_cost as f32, ctx.timestamp);
            ctx.db.health_state().entity_id().update(health);
        }
    }

    // Expend stamina
    if custom_effect.stamina_cost > 0 {
        let mut stamina = ctx.db.stamina_state().entity_id().find(actor_id).unwrap();
        if (stamina.stamina as i32) < custom_effect.stamina_cost {
            return Err("Not enough stamina to trigger this ability".into());
        }
        if !dry_run {
            stamina.stamina -= custom_effect.stamina_cost as f32;
            ctx.db.stamina_state().entity_id().update(stamina);
        }
    }

    // Validate distance
    let actor_location = game_state_filters::coordinates_float(ctx, actor_id);
    let target_location = game_state_filters::coordinates_float(ctx, target_entity_id);
    if actor_location.distance_to(target_location) > custom_effect.range + 3.0 {
        return Err("Too far".into());
    }

    // Validate target type
    // TODO - Enemies triggering abilities?
    if custom_effect.friendly {
        if ctx.db.player_state().entity_id().find(target_entity_id).is_none() {
            return Err("Invalid target".into());
        }
    } else {
        if ctx.db.enemy_state().entity_id().find(target_entity_id).is_none() {
            return Err("Invalid target".into());
        }
    }

    if dry_run {
        return Ok(());
    }

    // Apply cooldowns
    if !ability_state.set_cooldown(ctx, custom_effect.cooldown, custom_effect.global_cooldown, true) {
        return Err("Ability is under cooldown".into());
    }

    // TODO - Radius + friendly
    let targets = vec![target_entity_id];

    for target in targets {
        // Apply buffs
        if custom_effect.buffs.len() > 0 {
            let mut active_buff_state = unwrap_or_err!(
                ctx.db.active_buff_state().entity_id().find(target),
                "Entity has no ActiveBuffState."
            );
            for buff_effect in &custom_effect.buffs {
                if custom_effect.buff_toggle && active_buff_state.has_active_buff(buff_effect.buff_id, ctx.timestamp) {
                    // toggle of existing buff
                    active_buff_state.remove_active_buff(ctx, buff_effect.buff_id);
                } else {
                    // apply buff
                    active_buff_state.add_active_buff_with_data(ctx, buff_effect.buff_id, buff_effect.duration, None);
                }
            }
            ctx.db.active_buff_state().entity_id().update(active_buff_state);
        }

        // Apply damage / healing
        if custom_effect.damage != 0 {
            let mut health = ctx.db.health_state().entity_id().find(target).unwrap();
            health.add_health_delta(-custom_effect.health_cost as f32, ctx.timestamp);
            ctx.db.health_state().entity_id().update(health);
        }

        // Apply threat
        if custom_effect.threat_value != 0.0 {
            ThreatState::add_threat(ctx, actor_id, target, custom_effect.threat_value);
        }
    }

    // Apply Linked effect
    if custom_effect.linked_ability_buff_desc_id != 0 {
        let custom_effect = unwrap_or_err!(
            ctx.db.ability_custom_desc().id().find(custom_effect.linked_ability_buff_desc_id),
            "Unknown Linked Custom Effect"
        );
        return reduce(ctx, &custom_effect, target_entity_id, timestamp, dry_run);
    }

    Ok(())
}
