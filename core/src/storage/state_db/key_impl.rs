use db_key::Key;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct DBKey {
    data: Vec<u8>,
}

impl DBKey {
    pub fn new(data: Vec<u8>) -> Self {
        DBKey { data }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl AsRef<[u8]> for DBKey {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Ord for DBKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for DBKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DBKey {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for DBKey {}

impl Key for DBKey {
    fn from_u8(key: &[u8]) -> Self {
        DBKey { data: key.to_vec() }
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.data)
    }
}
