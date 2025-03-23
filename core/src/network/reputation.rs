use crate::types::node::Node;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ReputationManager {
    nodes: RwLock<HashMap<String, f32>>,
    decay_rate: f32,
}

impl ReputationManager {
    pub fn new(decay_rate: f32) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            decay_rate,
        }
    }

    /// Update reputation for a node
    pub fn update_reputation(&self, node_id: &str, delta: f32) {
        let mut nodes = self.nodes.write().unwrap();
        let entry = nodes.entry(node_id.to_string()).or_insert(1.0);
        *entry = (*entry + delta).clamp(0.0, 1.0);
    }

    /// Apply reputation decay based on time
    pub fn apply_decay(&self, node_id: &str, last_active: u64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let time_diff = current_time - last_active;
        let decay = self.decay_rate * time_diff as f32;

        let mut nodes = self.nodes.write().unwrap();
        if let Some(rep) = nodes.get_mut(node_id) {
            *rep = (*rep - decay).max(0.0);
        }
    }

    /// Get current reputation score
    pub fn get_reputation(&self, node_id: &str) -> f32 {
        let nodes = self.nodes.read().unwrap();
        *nodes.get(node_id).unwrap_or(&1.0)
    }

    /// Reset reputation for a node
    pub fn reset_reputation(&self, node_id: &str) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(node_id.to_string(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_updates() {
        let manager = ReputationManager::new(0.01);
        let node_id = "node1";

        // Test initial reputation
        assert_eq!(manager.get_reputation(node_id), 1.0);

        // Test positive update
        manager.update_reputation(node_id, 0.1);
        assert_eq!(manager.get_reputation(node_id), 1.0);

        // Test negative update
        manager.update_reputation(node_id, -0.5);
        assert_eq!(manager.get_reputation(node_id), 0.5);

        // Test decay
        manager.apply_decay(node_id, 0);
        assert!(manager.get_reputation(node_id) < 0.5);

        // Test reset
        manager.reset_reputation(node_id);
        assert_eq!(manager.get_reputation(node_id), 1.0);
    }
}
