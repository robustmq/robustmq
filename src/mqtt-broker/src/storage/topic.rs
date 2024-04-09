use super::{
    all::AllInfoStorage,
    keys::{all_topic_key, topic_key},
};
use crate::metadata::topic::Topic;
use common_base::errors::RobustMQError;
use std::{collections::HashMap, sync::Arc};
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct TopicStorage {
    storage_adapter: Arc<PlacementStorageAdapter>,
    all_info_storage: AllInfoStorage,
}

impl TopicStorage {
    pub fn new(storage_adapter: Arc<PlacementStorageAdapter>) -> Self {
        let all_info_storage = AllInfoStorage::new(all_topic_key(), storage_adapter.clone());
        return TopicStorage {
            storage_adapter,
            all_info_storage,
        };
    }
    // Persistence holds the session information of the connection dimension
    pub async fn save_topic(
        &self,
        topic_name: &String,
        topic: &Topic,
    ) -> Result<(), RobustMQError> {
        let key = topic_key(topic_name.clone());
        match serde_json::to_string(topic) {
            Ok(data) => {
                match self
                    .all_info_storage
                    .add_info_for_all(topic_name.clone())
                    .await
                {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                return self.storage_adapter.kv_set(key, data).await;
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    // Getting a list of users
    pub async fn topic_list(&self) -> Result<HashMap<String, Topic>, RobustMQError> {
        match self.all_info_storage.get_all().await {
            Ok(data) => {
                let mut list = HashMap::new();
                for username in data {
                    match self.get_topic(username.clone()).await {
                        Ok(user) => {
                            list.insert(username, user);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                return Ok(list);
            }
            Err(e) => return Err(e),
        }
    }

    // Get session information for the connection dimension
    pub async fn get_topic(&self, client_id: String) -> Result<Topic, RobustMQError> {
        let key = topic_key(client_id);
        match self.storage_adapter.kv_get(key).await {
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
