use crate::record::Record;
use axum::async_trait;
use common_base::errors::RobustMQError;

#[async_trait]
pub trait StorageAdapter {
    // kv storage model: Set data
    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError>;

    // kv storage model: Get data
    async fn get(&self, key: String) -> Result<Option<Record>, RobustMQError>;

    // kv storage model: Delete data
    async fn delete(&self, key: String) -> Result<(), RobustMQError>;

    // kv storage model: Determines whether the key exists
    async fn exists(&self, key: String) -> Result<bool, RobustMQError>;

    // Streaming storage model: Append data in a Shard dimension, returning a unique self-incrementing ID for the Shard dimension
    async fn stream_write(&self, shard_name: String, data: Vec<Record>) -> Result<Vec<usize>, RobustMQError>;

    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_read(
        &self,
        shard_name: String,
        group_id: String,
        record_num: Option<u128>,
        record_size: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError>;

    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_commit_offset(
        &self,
        shard_name: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, RobustMQError>;

    // Streaming storage model: A piece of data is uniquely read based on the shard name and a unique auto-incrementing ID.
    async fn stream_read_by_offset(
        &self,
        shard_name: String,
        record_id: usize,
    ) -> Result<Option<Record>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and time range.
    async fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
        record_num: Option<usize>,
        record_size: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and the last time it expires
    async fn stream_read_by_key(
        &self,
        shard_name: String,
        key: String,
    ) -> Result<Option<Record>, RobustMQError>;
}
