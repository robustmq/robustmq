use common_base::errors::RobustMQError;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct AllInfoStorage {
    pub key: String,
    storage_adapter: PlacementStorageAdapter,
}

impl AllInfoStorage {
    pub fn new(key: String) -> AllInfoStorage {
        let storage_adapter = PlacementStorageAdapter::new();
        return AllInfoStorage {
            key,
            storage_adapter,
        };
    }

    pub fn add_info_for_all(&self, item: String) -> Result<(), RobustMQError> {
        let mut all = match self.get_all() {
            Ok(da) => da,
            Err(_) => Vec::new(),
        };
        all.push(item);
        match serde_json::to_string(&all) {
            Ok(data) => return self.storage_adapter.kv_set(self.key.clone(), data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    pub fn remove_info_for_all(&self, topic_name: String) -> Result<(), RobustMQError> {
        let mut all_topic = match self.get_all() {
            Ok(da) => da,
            Err(_) => Vec::new(),
        };
        all_topic.push(topic_name);

        match serde_json::to_string(&all_topic) {
            Ok(data) => return self.storage_adapter.kv_set(self.key.clone(), data),
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(
                    e.to_string(),
                ))
            }
        }
    }

    pub fn get_all(&self) -> Result<Vec<String>, RobustMQError> {
        match self.storage_adapter.kv_get(self.key.clone()) {
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
