use common_base::errors::RobustMQError;

pub trait StorageAdapter {
    fn kv_set(&self, key: String, value: String) -> Result<(), RobustMQError>;
    fn kv_get(&self, key: String) -> Result<String, RobustMQError>;
    fn kv_delete(&self, key: String) -> Result<(), RobustMQError>;
    fn kv_exists(&self, key: String) -> Result<bool, RobustMQError>;
    fn stream_write(&self) -> Result<(), RobustMQError>;
    fn stream_read(&self) -> Result<String, RobustMQError>;
}
