use super::{
    all::AllInfoStorage,
    keys::{all_topic_key, topic_key},
};
use crate::metadata::topic::Topic;
use common_base::errors::RobustMQError;
use std::{collections::HashMap, sync::Arc};
use storage_adapter::{record::Record, storage::StorageAdapter};

pub struct TopicStorage<T> {
    storage_adapter: Arc<T>,
    all_info_storage: AllInfoStorage<T>,
}

impl<T> TopicStorage<T>
where
    T: StorageAdapter,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
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
        match serde_json::to_vec(topic) {
            Ok(data) => {
                match self
                    .all_info_storage
                    .add_info_for_all(topic_name.clone())
                    .await
                {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                return self.storage_adapter.set(key, Record::build_b(data)).await;
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save topic error, error messsage:{}",
                    e.to_string()
                )))
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
                            if let Some(t) = user {
                                list.insert(username, t);
                            }
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
    pub async fn get_topic(&self, client_id: String) -> Result<Option<Topic>, RobustMQError> {
        let key = topic_key(client_id);
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(Some(da));
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(format!(
                        "get topic config error, error messsage:{}",
                        e.to_string()
                    )))
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
