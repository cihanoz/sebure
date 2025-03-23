use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Regular validator node
    Validator,
    /// Supernode with enhanced capabilities
    Supernode,
    /// Lightweight client node
    LightClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub tier: u8,
    pub parent: Option<String>, // Parent node in hierarchy
    pub children: Vec<String>, // Child nodes
    pub reputation: f32,       // Reputation score (0.0 - 1.0)
    pub role: NodeRole,
    pub last_active: u64,      // Timestamp of last activity
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeRole {
    Validator,
    BlockProducer,
    NetworkRouter,
    DataShard,
    Observer,
}

impl Node {
    pub fn new(id: String, node_type: NodeType) -> Self {
        Self {
            id,
            node_type,
            tier: 0,
            parent: None,
            children: Vec::new(),
            reputation: 1.0,
            role: NodeRole::Observer,
            last_active: 0,
        }
    }

    pub fn is_supernode(&self) -> bool {
        matches!(self.node_type, NodeType::Supernode)
    }

    pub fn can_validate(&self) -> bool {
        matches!(self.role, NodeRole::Validator | NodeRole::BlockProducer)
    }
}
