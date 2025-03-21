//! # Database Iterator
//!
//! This module defines the iterator for the state database.

use leveldb::iterator::Iterator as LevelIterator;
use super::key_impl::DBKey;

/// Wrapper for LevelDB iterator
pub struct LevelDBIterator<'a> {
    pub inner: LevelIterator<'a, DBKey>,
}

impl<'a> Iterator for LevelDBIterator<'a> {
    type Item = (DBKey, Vec<u8>);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
