use super::keys::{all_topic_key, lastwill_key, session_key, topic_key};
use crate::metadata::{session::Session, topic::Topic};
use common_base::errors::RobustMQError;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

#[derive(Clone)]
pub struct MetadataStorage {
    storage_adapter: PlacementStorageAdapter,
}

impl MetadataStorage {
    pub fn new() -> Self {
        let storage_adapter = PlacementStorageAdapter::new();
        return MetadataStorage { storage_adapter };
    }

    // Persistence holds the session information of the connection dimension
    pub fn save_session(&self, client_id: String, session: Session) -> Result<(), RobustMQError> {
        let key = session_key(client_id);
        match serde_json::to_string(&session) {
            Ok(data) => return self.storage_adapter.kv_set(key, data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get session information for the connection dimension
    pub fn get_session(&self, client_id: String) -> Result<Session, RobustMQError> {
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

    // Persistence holds the session information of the connection dimension
    pub fn save_topic(&self, topic_name: &String, topic: &Topic) -> Result<(), RobustMQError> {
        let key = topic_key(topic_name.clone());
        match serde_json::to_string(topic) {
            Ok(data) => {
                match self.save_all_topic(topic_name.clone()) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                return self.storage_adapter.kv_set(key, data);
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get session information for the connection dimension
    pub fn get_topic(&self, client_id: String) -> Result<Topic, RobustMQError> {
        let key = topic_key(client_id);
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

    // Update the collection of all topics in the cluster
    pub fn save_all_topic(&self, topic_name: String) -> Result<(), RobustMQError> {
        let mut all_topic = match self.get_all_topic() {
            Ok(da) => da,
            Err(_) => Vec::new(),
        };
        all_topic.push(topic_name);
        let key = all_topic_key();
        match serde_json::to_string(&all_topic) {
            Ok(data) => return self.storage_adapter.kv_set(key, data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Get all the topics in the cluster
    pub fn get_all_topic(&self) -> Result<Vec<String>, RobustMQError> {
        let key = all_topic_key();
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
