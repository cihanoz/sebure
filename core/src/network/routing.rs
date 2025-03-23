use crate::types::node::{Node, NodeRole};
use crate::network::supernode::SupernodeManager;
use std::sync::Arc;

pub struct TieredRouter {
    supernode_manager: Arc<SupernodeManager>,
}

impl TieredRouter {
    pub fn new(supernode_manager: Arc<SupernodeManager>) -> Self {
        Self { supernode_manager }
    }

    /// Route message through the node hierarchy
    pub fn route(&self, source_tier: u8, target_tier: u8, message: Vec<u8>) {
        // Get all nodes in the source tier
        let source_nodes = self.supernode_manager.get_tier_routing(source_tier);
        
        // Find the best path between tiers
        let path = self.calculate_path(source_tier, target_tier);
        
        // Route through each node in the path
        for tier in path {
            let nodes = self.supernode_manager.get_tier_routing(tier);
            
            // Select appropriate nodes based on role
            let routers: Vec<Arc<Node>> = nodes.into_iter()
                .filter(|n| n.role == NodeRole::NetworkRouter)
                .collect();
                
            // TODO: Implement actual message passing
        }
    }

    /// Calculate optimal path between tiers
    fn calculate_path(&self, source: u8, target: u8) -> Vec<u8> {
        if source < target {
            (source..=target).collect()
        } else {
            (target..=source).rev().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::{Node, NodeType, NodeRole};
    use std::sync::Arc;

    #[test]
    fn test_tier_routing() {
        let manager = Arc::new(SupernodeManager::new());
        let router = TieredRouter::new(manager.clone());

        // Add test nodes
        let node1 = Node {
            id: "node1".to_string(),
            node_type: NodeType::Supernode,
            tier: 0,
            role: NodeRole::NetworkRouter,
            ..Node::new("node1".to_string(), NodeType::Supernode)
        };

        let node2 = Node {
            id: "node2".to_string(),
            node_type: NodeType::Supernode,
            tier: 1,
            role: NodeRole::NetworkRouter,
            ..Node::new("node2".to_string(), NodeType::Supernode)
        };

        manager.add_node(node1);
        manager.add_node(node2);

        // Test routing path calculation
        let path = router.calculate_path(0, 1);
        assert_eq!(path, vec![0, 1]);
    }
}
