pub mod cache_manager;
pub mod client_keep_alive;
pub mod connection;
pub mod error;
pub mod session;
pub mod topic;
pub mod retain_message;

pub const HEART_CONNECT_SHARD_HASH_NUM: u64 = 20;
