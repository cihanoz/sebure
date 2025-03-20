//! # Serialization Utilities
//! 
//! This module provides utilities for serializing and deserializing 
//! blockchain data structures in various formats.

use serde::{Serialize, Deserialize};
use crate::types::{Result, Error};

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// Binary format (using bincode)
    Binary,
    
    /// JSON format
    Json,
    
    /// Custom binary format (optimized for specific data structures)
    CustomBinary,
}

impl Default for SerializationFormat {
    fn default() -> Self {
        SerializationFormat::Binary
    }
}

/// Serialize a value into bytes using the specified format
pub fn serialize<T: Serialize>(value: &T, format: SerializationFormat) -> Result<Vec<u8>> {
    match format {
        SerializationFormat::Binary => {
            bincode::serialize(value)
                .map_err(|e| Error::Serialization(format!("Bincode serialization error: {}", e)))
        },
        
        SerializationFormat::Json => {
            serde_json::to_vec(value)
                .map_err(|e| Error::Serialization(format!("JSON serialization error: {}", e)))
        },
        
        SerializationFormat::CustomBinary => {
            // This would implement a custom, optimized binary format
            // For now, we just use bincode
            bincode::serialize(value)
                .map_err(|e| Error::Serialization(format!("Custom binary serialization error: {}", e)))
        },
    }
}

/// Deserialize bytes into a value using the specified format
pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8], format: SerializationFormat) -> Result<T> {
    match format {
        SerializationFormat::Binary => {
            bincode::deserialize(bytes)
                .map_err(|e| Error::Serialization(format!("Bincode deserialization error: {}", e)))
        },
        
        SerializationFormat::Json => {
            serde_json::from_slice(bytes)
                .map_err(|e| Error::Serialization(format!("JSON deserialization error: {}", e)))
        },
        
        SerializationFormat::CustomBinary => {
            // This would implement a custom, optimized binary format
            // For now, we just use bincode
            bincode::deserialize(bytes)
                .map_err(|e| Error::Serialization(format!("Custom binary deserialization error: {}", e)))
        },
    }
}

/// Serialize a value to a JSON string
pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map_err(|e| Error::Serialization(format!("JSON string serialization error: {}", e)))
}

/// Deserialize a JSON string into a value
pub fn from_json<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T> {
    serde_json::from_str(json)
        .map_err(|e| Error::Serialization(format!("JSON string deserialization error: {}", e)))
}

/// Calculate the size of a serialized value
pub fn serialized_size<T: Serialize>(value: &T, format: SerializationFormat) -> Result<usize> {
    let serialized = serialize(value, format)?;
    Ok(serialized.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestStruct {
        id: u32,
        name: String,
        data: Vec<u8>,
    }
    
    #[test]
    fn test_binary_serialization() {
        let test_struct = TestStruct {
            id: 42,
            name: "Test".to_string(),
            data: vec![1, 2, 3, 4],
        };
        
        // Test serialization
        let serialized = serialize(&test_struct, SerializationFormat::Binary).unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: TestStruct = deserialize(&serialized, SerializationFormat::Binary).unwrap();
        assert_eq!(deserialized, test_struct);
    }
    
    #[test]
    fn test_json_serialization() {
        let test_struct = TestStruct {
            id: 42,
            name: "Test".to_string(),
            data: vec![1, 2, 3, 4],
        };
        
        // Test serialization
        let serialized = serialize(&test_struct, SerializationFormat::Json).unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: TestStruct = deserialize(&serialized, SerializationFormat::Json).unwrap();
        assert_eq!(deserialized, test_struct);
    }
    
    #[test]
    fn test_json_string() {
        let test_struct = TestStruct {
            id: 42,
            name: "Test".to_string(),
            data: vec![1, 2, 3, 4],
        };
        
        // Test to_json
        let json_string = to_json(&test_struct).unwrap();
        assert!(json_string.contains("Test"));
        
        // Test from_json
        let deserialized: TestStruct = from_json(&json_string).unwrap();
        assert_eq!(deserialized, test_struct);
    }
    
    #[test]
    fn test_serialized_size() {
        let test_struct = TestStruct {
            id: 42,
            name: "Test".to_string(),
            data: vec![1, 2, 3, 4],
        };
        
        let binary_size = serialized_size(&test_struct, SerializationFormat::Binary).unwrap();
        let json_size = serialized_size(&test_struct, SerializationFormat::Json).unwrap();
        
        // JSON is typically larger than binary
        assert!(json_size > binary_size);
    }
}
