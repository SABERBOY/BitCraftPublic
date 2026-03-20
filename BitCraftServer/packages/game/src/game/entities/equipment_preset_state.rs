use crate::messages::components::EquipmentPresetState;

use crate::game::game_state::{self};
use crate::messages::components::*;
use crate::messages::static_data::{equipment_preset_knowledge_desc, EquipmentSlot, EquipmentSlotType};
use spacetimedb::{ReducerContext, Table};

impl EquipmentPresetState {
    // No longer a reducer
    pub fn on_knowledge_acquired(ctx: &ReducerContext, player_entity_id: u64, knowledge_id: i32) {
        if ctx.db.equipment_preset_knowledge_desc().knowledge_id().find(knowledge_id).is_some() {
            let nb_equipment_presets = ctx.db.equipment_preset_state().player_entity_id().filter(player_entity_id).count();

            // Add equipment presets for each unmapped knowledge
            let equipment_slots = vec![
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::MainHand, // Obsolete for now
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::OffHand, // Obsolete for now
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::HeadArtifact, // Seems to be where the Heart artifact is stored
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::TorsoArtifact, // Seems unused
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::HandArtifact, // Ring artifacts
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::FeetArtifact, // Seems unused
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::HeadClothing,
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::TorsoClothing,
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::HandClothing,
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::BeltClothing,
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::LegClothing,
                },
                EquipmentSlot {
                    item: None,
                    primary: EquipmentSlotType::FeetClothing,
                },
            ];
            let preset = EquipmentPresetState {
                entity_id: game_state::create_entity(ctx),
                player_entity_id,
                index: nb_equipment_presets as i32 + 1,
                active: false,
                equipment_slots,
            };
            ctx.db.equipment_preset_state().insert(preset);
        }
    }
}
