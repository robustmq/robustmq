use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use std::sync::Arc;

use crate::security::acl::MQTTAcl;

pub struct AclStorage {
    client_poll: Arc<ClientPool>,
}

impl AclStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return AclStorage { client_poll };
    }

    pub fn list_acl() -> Result<Vec<MQTTAcl>, RobustMQError> {
        let list = Vec::new();
        
        return Ok(list);
    }
}
