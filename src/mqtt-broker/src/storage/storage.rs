use common_base::errors::RobustMQError;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

use crate::metadata::session::{LastWillData, Session};

use super::keys::{lastwill_key, session_key};

#[derive(Clone)]
pub struct StorageLayer {
    storage_adapter: PlacementStorageAdapter,
}

impl StorageLayer {
    pub fn new() -> Self {
        let storage_adapter = PlacementStorageAdapter::new();
        return StorageLayer { storage_adapter };
    }

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
}
