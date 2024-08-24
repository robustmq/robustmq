use clients::{placement::mqtt::call::list_acl, poll::ClientPool};
use common_base::{config::broker_mqtt::broker_mqtt_conf, error::common::CommonError};
use metadata_struct::acl::mqtt_acl::MQTTAcl;
use protocol::placement_center::generate::mqtt::ListAclRequest;
use std::sync::Arc;

pub struct AclStorage {
    client_poll: Arc<ClientPool>,
}

impl AclStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return AclStorage { client_poll };
    }

    pub async fn list_acl(&self) -> Result<Vec<MQTTAcl>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListAclRequest {
            cluster_name: config.cluster_name.clone(),
        };
        match list_acl(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let mut list = Vec::new();
                for raw in reply.acls {
                    list.push(serde_json::from_slice::<MQTTAcl>(raw.as_slice())?);
                }
                return Ok(list);
            }
            Err(e) => return Err(e),
        }
    }
}
