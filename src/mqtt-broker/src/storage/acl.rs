use clients::poll::ClientPool;
use common_base::error::common::CommonError;
use std::sync::Arc;

use crate::security::acl::MQTTAcl;

pub struct AclStorage {
    client_poll: Arc<ClientPool>,
}

impl AclStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return AclStorage { client_poll };
    }

    pub fn list_acl() -> Result<Vec<MQTTAcl>, CommonError> {
        let list = Vec::new();
        
        return Ok(list);
    }
}
