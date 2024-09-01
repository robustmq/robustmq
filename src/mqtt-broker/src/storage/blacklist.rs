use clients::{placement::mqtt::call::list_blacklist, poll::ClientPool};
use common_base::{config::broker_mqtt::broker_mqtt_conf, error::common::CommonError};
use metadata_struct::acl::mqtt_blacklist::MQTTAclBlackList;
use protocol::placement_center::generate::mqtt::ListBlacklistRequest;
use std::sync::Arc;

pub struct BlackListStorage {
    client_poll: Arc<ClientPool>,
}

impl BlackListStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return BlackListStorage { client_poll };
    }

    pub async fn list_blacklist(&self) -> Result<Vec<MQTTAclBlackList>, CommonError> {
        let config = broker_mqtt_conf();
        let request = ListBlacklistRequest {
            cluster_name: config.cluster_name.clone(),
        };
        match list_blacklist(
            self.client_poll.clone(),
            config.placement_center.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let mut list = Vec::new();
                for raw in reply.blacklists {
                    list.push(serde_json::from_slice::<MQTTAclBlackList>(raw.as_slice())?);
                }
                return Ok(list);
            }
            Err(e) => return Err(e),
        }
    }
}
