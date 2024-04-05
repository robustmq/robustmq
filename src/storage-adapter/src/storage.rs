use common_base::errors::RobustMQError;

pub trait StorageAdapter {
    // kv storage model: Set data
    fn kv_set(&self, key: String, value: String) -> Result<(), RobustMQError>;
    // kv storage model: Get data
    fn kv_get(&self, key: String) -> Result<String, RobustMQError>;
    // kv storage model: Delete data
    fn kv_delete(&self, key: String) -> Result<(), RobustMQError>;
    // kv storage model: Determines whether the key exists
    fn kv_exists(&self, key: String) -> Result<bool, RobustMQError>;
    // Streaming storage model: Append data in a Shard dimension, returning a unique self-incrementing ID for the Shard dimension
    fn stream_write(&self, shard_name: String, bytess: Vec<u8>) -> Result<u128, RobustMQError>;
    // Streaming storage model: Read the next item in the dimension of the Shard + subscription name tuple
    fn stream_read_next(&self, shard_name: String) -> Result<Vec<u8>, RobustMQError>;
    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    fn stream_read_next_batch(
        &self,
        shard_name: String,
        record_num: u16,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;
    // Streaming storage model: A piece of data is uniquely read based on the shard name and a unique auto-incrementing ID.
    fn stream_read_by_id(
        &self,
        shard_name: String,
        record_id: u128,
    ) -> Result<Vec<u8>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and time range.
    fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;

    // Streaming storage model: A batch of data is read based on the shard name and the last time it expires
    fn stream_read_by_last_expiry(
        &self,
        shard_name: String,
        second: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError>;
}
