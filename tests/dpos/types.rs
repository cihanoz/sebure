// Basic types
pub type BlockHeight = u64;
pub type ShardId = u16;
pub type Timestamp = u64;

// Error handling
#[derive(Debug)]
pub enum Error {
    Consensus(String),
    BlockValidation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
