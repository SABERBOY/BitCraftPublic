use crate::messages::components::{KnowledgeSecondaryState, KnowledgeState};

impl KnowledgeSecondaryState {
    pub fn is_acquired(&self, secondary_id: i32) -> bool {
        if let Some(entry) = self.entries.iter().find(|e| e.id == secondary_id) {
            // acquired or discovered means discovered
            return entry.state == KnowledgeState::Acquired;
        }
        false
    }
}
