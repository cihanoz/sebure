//! # Utility Modules
//! 
//! This module contains various utility functions and helpers
//! used throughout the SEBURE blockchain.

pub mod serialization;

// Re-export commonly used utilities for easier access
pub use serialization::{
    serialize, 
    deserialize, 
    to_json, 
    from_json, 
    serialized_size, 
    SerializationFormat
};
