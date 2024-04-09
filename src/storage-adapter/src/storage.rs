use axum::async_trait;
use common_base::errors::RobustMQError;

#[async_trait]
pub trait StorageAdapter {
    // kv storage model: Set data
    async fn  kv_set(&self, key: String, value: String) -> Result<(), RobustMQError>;
    // kv storage model: Get data
    async fn kv_get(&self, key: String) -> Result<String, RobustMQError>;
    // kv storage model: Delete data
    async fn kv_delete(&self, key: String) -> Result<(), RobustMQError>;
    // kv storage model: Determines whether the key exists
    async fn kv_exists(&self, key: String) -> Result<bool, RobustMQError>;
    // Streaming storage model: Append data in a Shard dimension, returning a unique self-incrementing ID for the Shard dimension
    async fn stream_write(&self, shard_name: String, bytess: Vec<u8>) -> Result<u128, RobustMQError>;
    // Streaming storage model: Read the next item in the dimension of the Shard + subscription name tuple
    async fn stream_read_next(&self, shard_name: String) -> Result<Vec<u8>, RobustMQError>;
    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_read_next_batch(
        &self,
        shard_name: String,
        record_num: u16,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;
    // Streaming storage model: A piece of data is uniquely read based on the shard name and a unique auto-incrementing ID.
    async fn stream_read_by_id(
        &self,
        shard_name: String,
        record_id: u128,
    ) -> Result<Vec<u8>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and time range.
    async fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and the last time it expires
    async fn stream_read_by_last_expiry(
        &self,
        shard_name: String,
        second: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;
}
