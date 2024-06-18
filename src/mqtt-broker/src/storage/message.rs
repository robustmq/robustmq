use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::{record::Record, storage::StorageAdapter};

#[derive(Clone)]
pub struct MessageStorage<T> {
    storage_adapter: Arc<T>,
}

impl<T> MessageStorage<T>
where
    T: StorageAdapter + Send + Sync + 'static,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
        return MessageStorage { storage_adapter };
    }

    // Save the data for the Topic dimension
    pub async fn append_topic_message(
        &self,
        topic_id: String,
        record: Vec<Record>,
    ) -> Result<Vec<usize>, RobustMQError> {
        let shard_name = topic_id;
        match self.storage_adapter.stream_write(shard_name, record).await {
            Ok(id) => {
                return Ok(id);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Read the data for the Topic dimension
    pub async fn read_topic_message(
        &self,
        topic_id: String,
        group_id: String,
        record_num: u128,
    ) -> Result<Vec<Record>, RobustMQError> {
        let shard_name = topic_id;
        match self
            .storage_adapter
            .stream_read(shard_name, group_id, Some(record_num), None)
            .await
        {
            Ok(Some(data)) => {
                return Ok(data);
            }
            Ok(None) => {
                return Ok(Vec::new());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Submits the offset information for consumption
    pub async fn commit_group_offset(
        &self,
        topic_id: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, RobustMQError> {
        let shard_name = topic_id;
        match self
            .storage_adapter
            .stream_commit_offset(shard_name, group_id, offset)
            .await
        {
            Ok(flag) => {
                return Ok(flag);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
