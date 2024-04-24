use super::keys::{lastwill_key, session_key};
use crate::metadata::session::Session;
use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::{record::Record, storage::StorageAdapter};

pub struct SessionStorage<T> {
    storage_adapter: Arc<T>,
}

impl<T> SessionStorage<T>
where
    T: StorageAdapter,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
        return SessionStorage { storage_adapter };
    }

    // Persistence holds the session information of the connection dimension
    pub async fn save_session(
        &self,
        client_id: String,
        session: &Session,
    ) -> Result<(), RobustMQError> {
        let key = session_key(client_id);
        match serde_json::to_vec(session) {
            Ok(data) => return self.storage_adapter.set(key, Record::build_b(data)).await,
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save session error, error messsage:{}",
                    e.to_string()
                )))
            }
        }
    }

    // Get session information for the connection dimension
    pub async fn get_session(&self, client_id: &String) -> Result<Option<Session>, RobustMQError> {
        let key = lastwill_key(client_id.clone());
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(format!(
                        "get session error, error messsage:{}",
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
