use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::{record::Record, storage::StorageAdapter};

pub struct AllInfoStorage<T> {
    pub key: String,
    storage_adapter: Arc<T>,
}

impl<T> AllInfoStorage<T>
where
    T: StorageAdapter,
{
    pub fn new(key: String, storage_adapter: Arc<T>) -> AllInfoStorage<T> {
        return AllInfoStorage {
            key,
            storage_adapter,
        };
    }

    pub async fn add_info_for_all(&self, item: String) -> Result<(), RobustMQError> {
        let mut all = match self.get_all().await {
            Ok(da) => da,
            Err(_) => Vec::new(),
        };
        all.push(item);
        match serde_json::to_vec(&all) {
            Ok(data) => {
                return self
                    .storage_adapter
                    .set(self.key.clone(), Record::build_b(data))
                    .await
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    pub async fn remove_info_for_all(&self, topic_name: String) -> Result<(), RobustMQError> {
        let mut all_topic = match self.get_all().await {
            Ok(da) => da,
            Err(_) => Vec::new(),
        };
        all_topic.push(topic_name);

        match serde_json::to_vec(&all_topic) {
            Ok(data) => {
                return self
                    .storage_adapter
                    .set(self.key.clone(), Record::build_b(data))
                    .await
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    pub async fn get_all(&self) -> Result<Vec<String>, RobustMQError> {
        match self.storage_adapter.exists(self.key.clone()).await {
            Ok(flag) => {
                if !flag {
                    return Ok(Vec::new());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        match self.storage_adapter.get(self.key.clone()).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(format!(
                        "get call data error, error messsage:{}",
                        e.to_string()
                    )))
                }
            },
            Ok(None) => {
                return Ok(Vec::new());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
