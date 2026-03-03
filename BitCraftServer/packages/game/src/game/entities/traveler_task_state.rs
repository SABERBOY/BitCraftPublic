use std::collections::HashMap;

use spacetimedb::{log, rand::Rng, ReducerContext, Table};

use crate::{
    game::game_state,
    messages::{
        components::{experience_state, knowledge_secondary_state, player_state, traveler_task_state, TravelerTaskState},
        static_data::{
            npc_desc, traveler_task_desc, traveler_task_knowledge_requirement_desc, TravelerTaskDesc, TravelerTaskKnowledgeRequirementDesc,
        },
    },
};

impl TravelerTaskState {
    pub fn delete_all_for_player(ctx: &ReducerContext, player_entity_id: u64) {
        ctx.db.traveler_task_state().player_entity_id().delete(player_entity_id);
    }

    pub fn create_and_commit(ctx: &ReducerContext, player_entity_id: u64, traveler_id: i32, task_id: i32) {
        let entity_id = game_state::create_entity(ctx);
        let traveler_task = TravelerTaskState {
            entity_id,
            player_entity_id,
            traveler_id,
            task_id,
            completed: false,
        };
        ctx.db.traveler_task_state().insert(traveler_task);
    }

    pub fn generate_npc_requests_hashmap(ctx: &ReducerContext) -> HashMap<i32, Vec<TravelerTaskDesc>> {
        let mut npc_requests = HashMap::new();
        for npc in ctx.db.npc_desc().iter() {
            let npc_tasks = ctx
                .db
                .traveler_task_desc()
                .iter()
                .filter(|t| npc.task_skill_check.contains(&t.level_requirement.skill_id))
                .collect();
            npc_requests.insert(npc.npc_type, npc_tasks);
        }
        npc_requests
    }

    pub fn generate_all_for_player(
        ctx: &ReducerContext,
        player_entity_id: u64,
        requests: &HashMap<i32, Vec<TravelerTaskDesc>>,
        tasks_per_npc: i32,
        next_traveler_task_refresh: i32,
    ) {
        let mut player = ctx.db.player_state().entity_id().find(player_entity_id).unwrap();
        player.traveler_tasks_expiration = next_traveler_task_refresh;
        ctx.db.player_state().entity_id().update(player);

        let player_knowledges = ctx.db.knowledge_secondary_state().entity_id().find(player_entity_id).unwrap();
        let knowledge_requirements: HashMap<i32, TravelerTaskKnowledgeRequirementDesc> = ctx
            .db
            .traveler_task_knowledge_requirement_desc()
            .iter()
            .map(|requirements| (requirements.traveler_task_id, requirements))
            .collect();

        let experience = ctx.db.experience_state().entity_id().find(player_entity_id).unwrap();
        for traveler_id in requests.keys() {
            if requests[traveler_id].len() == 0 {
                continue;
            }
            let mut skill_appropriate_task_pool: Vec<i32> = requests[traveler_id]
                .iter()
                .filter(|t| {
                    let level = experience.get_level(t.level_requirement.skill_id);
                    level >= t.level_requirement.min_level && level <= t.level_requirement.max_level
                })
                .filter(|t| match knowledge_requirements.get(&t.id) {
                    Some(requirements) => {
                        requirements.required_knowledges.is_empty()
                            || requirements
                                .required_knowledges
                                .iter()
                                .all(|knowledge_id| player_knowledges.is_acquired(*knowledge_id))
                    }
                    None => true,
                })
                .filter(|t| match knowledge_requirements.get(&t.id) {
                    Some(requirements) => {
                        requirements.blocking_knowledges.is_empty()
                            || !requirements
                                .blocking_knowledges
                                .iter()
                                .any(|knowledge_id| player_knowledges.is_acquired(*knowledge_id))
                    }
                    None => true,
                })
                .map(|t| t.id)
                .collect();
            let iterations = skill_appropriate_task_pool.len().min(tasks_per_npc as usize);
            if iterations < tasks_per_npc as usize {
                log::error!("Player {player_entity_id} has only {iterations} tasks available for npc {traveler_id}");
            }

            for _i in 0..iterations {
                let rnd = ctx.rng().gen_range(0..skill_appropriate_task_pool.len());
                let task_id = skill_appropriate_task_pool.swap_remove(rnd);
                Self::create_and_commit(ctx, player_entity_id, *traveler_id, task_id);
            }
        }
    }
}
