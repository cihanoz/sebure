//! # State Database Tests
//!
//! This module contains tests for the state database.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::types::Error;
    use std::env;
    
    fn temp_dir() -> String {
        let mut dir = env::temp_dir();
        dir.push(format!("sebure-test-state-{}", rand::random::<u64>()));
        dir.to_str().unwrap().to_string()
    }
    
    #[test]
    fn test_leveldb_backend() {
        // Only run this test if not in test mode (which would use memory backend)
        if !cfg!(test) {
            let path = temp_dir();
            let config = super::super::super::StorageConfig::default();
            
            // Force LevelDB backend
            let state_db = state_db::StateDB {
                path: path.clone(),
                backend: database_types::DatabaseBackend::LevelDB,
                level_dbs: None,
                lmdb_env: None,
                lmdb_dbs: None,
                memory_storage: None,
                version: 1,
            };
            
            let result = state_db::StateDB::init_leveldb(&path, &config);
            assert!(result.is_ok());
            
            // Clean up
            std::fs::remove_dir_all(path).ok();
        }
    }
    
    #[test]
    fn test_lmdb_backend() {
        // Only run this test if not in test mode (which would use memory backend)
        if !cfg!(test) {
            let path = temp_dir();
            let config = super::super::super::StorageConfig::default();
            
            // Force LMDB backend
            let state_db = state_db::StateDB {
                path: path.clone(),
                backend: database_types::DatabaseBackend::LMDB,
                level_dbs: None,
                lmdb_env: None,
                lmdb_dbs: None,
                memory_storage: None,
                version: 1,
            };
            
            let result = state_db::StateDB::init_lmdb(&path, &config);
            assert!(result.is_ok());
            
            // Clean up
            std::fs::remove_dir_all(path).ok();
        }
    }
}
