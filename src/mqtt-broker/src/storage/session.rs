use super::keys::{lastwill_key, session_key};
use crate::metadata::session::Session;
use common_base::errors::RobustMQError;
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct SessionStorage {
    storage_adapter: PlacementStorageAdapter,
}

impl SessionStorage {
    pub fn new() -> Self {
        let storage_adapter = PlacementStorageAdapter::new();
        return SessionStorage { storage_adapter };
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
}
