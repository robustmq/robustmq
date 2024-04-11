use axum::async_trait;
use common_base::errors::RobustMQError;

#[async_trait]
pub trait StorageAdapter {
    // kv storage model: Set data
    async fn kv_set(&self, key: String, value: String) -> Result<(), RobustMQError>;
    
    // kv storage model: Get data
    async fn kv_get(&self, key: String) -> Option<String>;
    
    // kv storage model: Delete data
    async fn kv_delete(&self, key: String) -> Result<(), RobustMQError>;
    
    // kv storage model: Determines whether the key exists
    async fn kv_exists(&self, key: String) -> Result<bool, RobustMQError>;
    
    // Streaming storage model: Append data in a Shard dimension, returning a unique self-incrementing ID for the Shard dimension
    async fn stream_write(
        &self,
        shard_name: String,
        bytes: Vec<u8>,
    ) -> Result<usize, RobustMQError>;
    
    // Streaming storage model: Read the next item in the dimension of the Shard + subscription name tuple
    async fn stream_read_next(
        &self,
        shard_name: String,
        group_id: String,
    ) -> Result<Option<Vec<u8>>, RobustMQError>;

    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_read_next_batch(
        &self,
        shard_name: String,
        group_id: String,
        record_num: usize,
    ) -> Result<Option<Vec<Vec<u8>>>, RobustMQError>;

    // Streaming storage model: A piece of data is uniquely read based on the shard name and a unique auto-incrementing ID.
    async fn stream_read_by_offset(
        &self,
        shard_name: String,
        record_id: usize,
    ) -> Result<Option<Vec<u8>>, RobustMQError> ;

    // Streaming storage model: A batch of data is read based on the shard name and time range.
    async fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Option<Vec<Vec<u8>>>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and the last time it expires
    async fn stream_read_by_key(
        &self,
        shard_name: String,
        key: String,
    ) -> Result<Option<Vec<u8>>, RobustMQError>;
}
