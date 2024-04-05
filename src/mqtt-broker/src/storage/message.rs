use super::keys::{lastwill_key, retain_message};
use crate::metadata::{message::Message, session::LastWillData};
use common_base::errors::RobustMQError;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

#[derive(Clone)]
pub struct MessageStorage {
    storage_adapter: PlacementStorageAdapter,
}

impl MessageStorage {
    pub fn new() -> Self {
        let storage_adapter = PlacementStorageAdapter::new();
        return MessageStorage { storage_adapter };
    }

    // Save the data for the Topic dimension
    pub fn append_topic_message(
        &self,
        topic_id: String,
        message: Message,
    ) -> Result<u128, RobustMQError> {
        match serde_json::to_string(&message) {
            Ok(data) => {
                let shard_name = topic_id;
                match self
                    .storage_adapter
                    .stream_write(shard_name, data.into_bytes())
                {
                    Ok(id) => {
                        return Ok(id);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Read the data for the Topic dimension
    pub fn read_topic_message(
        &self,
        topic_id: String,
        record_num: u16,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        let shard_name = topic_id;
        return self
            .storage_adapter
            .stream_read_next_batch(shard_name, record_num);
    }

    // Read the data according to the message ID
    pub fn read_topic_message_by_id(
        &self,
        topic_id: String,
        record_id: u128,
    ) -> Result<Vec<u8>, RobustMQError> {
        let shard_name = topic_id;
        return self
            .storage_adapter
            .stream_read_by_id(shard_name, record_id);
    }

    // Saves the most recent reserved message for the Topic dimension
    pub fn save_retain_message(
        &self,
        topic_id: String,
        message: Message,
    ) -> Result<(), RobustMQError> {
        let key = retain_message(topic_id);
        match serde_json::to_string(&message) {
            Ok(data) => return self.storage_adapter.kv_set(key, data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get the latest reserved message for the Topic dimension
    pub fn get_retain_message(&self, topic_id: String) -> Result<Message, RobustMQError> {
        let key = retain_message(topic_id);
        match self.storage_adapter.kv_get(key) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Persistence holds the will message of the connection dimension
    pub fn save_lastwill(
        &self,
        client_id: String,
        last_will_data: LastWillData,
    ) -> Result<(), RobustMQError> {
        let key = lastwill_key(client_id);
        match serde_json::to_string(&last_will_data) {
            Ok(data) => return self.storage_adapter.kv_set(key, data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get the will message of the connection dimension
    pub fn get_lastwill(&self, client_id: String) -> Result<LastWillData, RobustMQError> {
        let key = lastwill_key(client_id);
        match self.storage_adapter.kv_get(key) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}
