use crate::types::node::{Node, NodeType, NodeRole};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct SupernodeManager {
    nodes: RwLock<HashMap<String, Arc<Node>>>,
    hierarchy: RwLock<HashMap<String, Vec<String>>>,
    routing_table: RwLock<HashMap<u8, HashSet<String>>>, // Tier -> Node IDs
}

impl SupernodeManager {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            hierarchy: RwLock::new(HashMap::new()),
            routing_table: RwLock::new(HashMap::new()),
        }
    }

    /// Add a new node to the hierarchy
    pub fn add_node(&self, node: Node) {
        let mut nodes = self.nodes.write().unwrap();
        let mut hierarchy = self.hierarchy.write().unwrap();
        let mut routing_table = self.routing_table.write().unwrap();

        let node_id = node.id.clone();
        let node = Arc::new(node);
        
        // Add to nodes map
        nodes.insert(node_id.clone(), node.clone());

        // Add to hierarchy
        if let Some(parent_id) = &node.parent {
            hierarchy.entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // Add to routing table by tier
        routing_table.entry(node.tier)
            .or_insert_with(HashSet::new)
            .insert(node_id);
    }

    /// Promote a node to supernode status
    pub fn promote_to_supernode(&self, node_id: &str) -> Result<(), String> {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(node_id) {
            let mut node = Arc::make_mut(node);
            node.node_type = NodeType::Supernode;
            Ok(())
        } else {
            Err("Node not found".to_string())
        }
    }

    /// Get all child nodes for a given parent
    pub fn get_children(&self, parent_id: &str) -> Vec<Arc<Node>> {
        let hierarchy = self.hierarchy.read().unwrap();
        let nodes = self.nodes.read().unwrap();

        hierarchy.get(parent_id)
            .map(|children| {
                children.iter()
                    .filter_map(|id| nodes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get routing information for a specific tier
    pub fn get_tier_routing(&self, tier: u8) -> Vec<Arc<Node>> {
        let routing_table = self.routing_table.read().unwrap();
        let nodes = self.nodes.read().unwrap();

        routing_table.get(&tier)
            .map(|node_ids| {
                node_ids.iter()
                    .filter_map(|id| nodes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Update node reputation
    pub fn update_reputation(&self, node_id: &str, delta: f32) -> Result<(), String> {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(node_id) {
            let mut node = Arc::make_mut(node);
            node.reputation = (node.reputation + delta).clamp(0.0, 1.0);
            Ok(())
        } else {
            Err("Node not found".to_string())
        }
    }

    /// Get all nodes in the network
    pub fn get_all_nodes(&self) -> Vec<Arc<Node>> {
        let nodes = self.nodes.read().unwrap();
        nodes.values().cloned().collect()
    }

    /// Update a node's role
    pub fn update_role(&self, node_id: &str, role: NodeRole) -> Result<(), String> {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(node_id) {
            let mut node = Arc::make_mut(node);
            node.role = role;
            Ok(())
        } else {
            Err("Node not found".to_string())
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<Arc<Node>> {
        let nodes = self.nodes.read().unwrap();
        nodes.get(node_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::{Node, NodeType};

    #[test]
    fn test_node_hierarchy() {
        let manager = SupernodeManager::new();
        
        let parent = Node::new("parent".to_string(), NodeType::Supernode);
        let child = Node {
            id: "child".to_string(),
            parent: Some("parent".to_string()),
            ..Node::new("child".to_string(), NodeType::Validator)
        };

        manager.add_node(parent);
        manager.add_node(child);

        let children = manager.get_children("parent");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child");
    }
}
