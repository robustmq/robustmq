use super::keys::{lastwill_key, retain_message};
use crate::metadata::{message::Message as RetainMessage, session::LastWillData};
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

    // Saves the most recent reserved message for the Topic dimension
    pub async fn save_retain_message(
        &self,
        topic_id: String,
        retail_message: RetainMessage,
    ) -> Result<(), RobustMQError> {
        let key = retain_message(topic_id);
        match serde_json::to_vec(&retail_message) {
            Ok(data) => return self.storage_adapter.set(key, Record::build_b(data)).await,
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get the latest reserved message for the Topic dimension
    pub async fn get_retain_message(
        &self,
        topic_id: String,
    ) -> Result<Option<RetainMessage>, RobustMQError> {
        let key = retain_message(topic_id);
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

}
