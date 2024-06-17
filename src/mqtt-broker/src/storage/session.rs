use super::keys::{lastwill_key, session_key};
use crate::metadata::session::Session;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::record::Record;

pub struct SessionStorage {
    client_poll: Arc<ClientPool>,
}

impl SessionStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return SessionStorage { client_poll };
    }

    // Persistence holds the session information of the connection dimension
    pub async fn save_session(
        &self,
        client_id: String,
        session: &Session,
    ) -> Result<(), RobustMQError> {
        return Ok(());
    }

    // Get session information for the connection dimension
    pub async fn get_session(&self, client_id: &String) -> Result<Option<Session>, RobustMQError> {
        return Ok(None);
    }
}
