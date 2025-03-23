use crate::types::node::{Node, NodeRole, NodeType};
use crate::network::supernode::SupernodeManager;
use std::sync::Arc;

pub struct RoleAssigner {
    supernode_manager: Arc<SupernodeManager>,
    min_reputation: f32,
}

impl RoleAssigner {
    pub fn new(supernode_manager: Arc<SupernodeManager>, min_reputation: f32) -> Self {
        Self {
            supernode_manager,
            min_reputation,
        }
    }

    /// Assign roles to nodes based on their characteristics
    pub fn assign_roles(&self) {
        let nodes = self.supernode_manager.get_all_nodes();
        
        for node in nodes {
            let role = self.determine_role(&node);
            self.supernode_manager.update_role(&node.id, role).unwrap();
        }
    }

    fn determine_role(&self, node: &Node) -> NodeRole {
        match node.node_type {
            NodeType::Supernode => {
                if node.tier == 0 {
                    NodeRole::BlockProducer
                } else {
                    NodeRole::NetworkRouter
                }
            }
            NodeType::Validator => {
                if node.reputation >= self.min_reputation {
                    NodeRole::Validator
                } else {
                    NodeRole::Observer
                }
            }
            NodeType::LightClient => NodeRole::Observer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::{Node, NodeType};
    use std::sync::Arc;

    #[test]
    fn test_role_assignment() {
        let supernode_manager = Arc::new(SupernodeManager::new());
        let role_assigner = RoleAssigner::new(supernode_manager.clone(), 0.8);

        let supernode = Node {
            id: "supernode".to_string(),
            node_type: NodeType::Supernode,
            tier: 0,
            ..Node::new("supernode".to_string(), NodeType::Supernode)
        };

        let validator = Node {
            id: "validator".to_string(),
            node_type: NodeType::Validator,
            reputation: 0.9,
            ..Node::new("validator".to_string(), NodeType::Validator)
        };

        supernode_manager.add_node(supernode);
        supernode_manager.add_node(validator);

        role_assigner.assign_roles();

        let supernode = supernode_manager.get_node("supernode").unwrap();
        let validator = supernode_manager.get_node("validator").unwrap();

        assert_eq!(supernode.role, NodeRole::BlockProducer);
        assert_eq!(validator.role, NodeRole::Validator);
    }
}
