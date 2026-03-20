use std::collections::HashMap;

use spacetimedb::ReducerContext;

use crate::messages::components::{equipment_preset_state, EquipmentState};
use crate::messages::static_data::*;
use crate::InventoryState;

impl EquipmentState {
    pub fn collect_stats(&self, ctx: &ReducerContext, bonuses: &mut HashMap<CharacterStatType, (f32, f32)>) {
        let mut equipped_item_ids = Vec::new();

        let active_preset = ctx
            .db
            .equipment_preset_state()
            .player_entity_id()
            .filter(self.entity_id)
            .find(|preset| preset.active);
        let using_active_preset = active_preset.is_some();

        let equipment_slots = if active_preset.is_some() {
            &active_preset.unwrap().equipment_slots
        } else {
            &self.equipment_slots
        };

        // collect item ids from equipped gear (extra check to avoid doubling equipment taking 2 slots, although this is no longer used outside character customization)
        for i in 0..equipment_slots.len() {
            let equipment_slot = &equipment_slots[i];
            if equipment_slot.item_id() > 0 && equipment_slot.primary as usize == i {
                equipped_item_ids.push(equipment_slot.item_id());
            }
        }

        // if using an equipment preset stats, we still need to collect the heart stats from the main equipment
        if using_active_preset {
            let equipment_slot = &self.equipment_slots[EquipmentSlotType::HeadArtifact as usize];
            if equipment_slot.item_id() > 0 {
                equipped_item_ids.push(equipment_slot.item_id());
            }
        }

        // collect toolbelt item ids
        let toolbelt_inv = InventoryState::get_player_toolbelt(ctx, self.entity_id).unwrap();
        for p in toolbelt_inv.pockets {
            if let Some(content) = p.contents {
                equipped_item_ids.push(content.item_id);
            }
        }

        // apply all equipped item stats
        for item_id in equipped_item_ids {
            if let Some(equipment) = ctx.db.equipment_desc().item_id().find(&item_id) {
                for stat_delta in &equipment.stats {
                    let entry = bonuses.entry(stat_delta.id).or_insert((0.0, 0.0));
                    if stat_delta.is_pct {
                        *entry = (entry.0, entry.1 + stat_delta.value);
                    } else {
                        *entry = (entry.0 + stat_delta.value, entry.1);
                    }
                }
            }
        }
    }

    /*
    pub fn get_weapon(&self, weapon_requirements: &Vec<WeaponRequirement>) -> Option<WeaponDesc> {
        let equipment_slots = &self.equipment_slots;
        let main_hand_slot = &equipment_slots[EquipmentSlotType::MainHand as usize];
        let off_hand_slot = &equipment_slots[EquipmentSlotType::OffHand as usize];
        for req in weapon_requirements {
            // check main hand slot
            if main_hand_slot.item_id() > 0 {
                if let Some(weapon) = ctx.db.weapon_desc().item_id().find(&main_hand_slot.item_id()) {
                    if req.weapon_type == weapon.weapon_type {
                        return Some(weapon.clone());
                    }
                }
            }

            // check off-hand slot
            if off_hand_slot.item_id() > 0 {
                if let Some(weapon) = ctx.db.weapon_desc().item_id().find(&off_hand_slot.item_id()) {
                    if req.weapon_type == weapon.weapon_type {
                        return Some(weapon.clone());
                    }
                }
            }
        }
        None
    }

    pub fn meet_requirement(&self, weapon_requirements: &Vec<WeaponRequirement>) -> bool {
        if self.get_weapon(weapon_requirements).is_some() {
            return true;
        }
        let equipment_slots = &self.equipment_slots;
        let main_hand_slot = &equipment_slots[EquipmentSlotType::MainHand as usize];
        main_hand_slot.item_id() == 0 && weapon_requirements.iter().any(|r| r.weapon_type == 0)
    }
    */
    /*
    pub fn meet_requirement(&self, weapon_requirements: &Vec<WeaponRequirement>) -> bool {
        if let Some(tool) = ToolDesc::get_best_weapon(self.entity_id) {
            if let Some(weapon) = ctx.db.weapon_desc().item_id().find(&tool.item_id) {
                return weapon_requirements.iter().any(|r| r.weapon_type == weapon.weapon_type);
            }
        }
        if weapon_requirements.iter().all(|w| w.weapon_type == 0) {
            return true;
        }

        false
    }
    */
}
