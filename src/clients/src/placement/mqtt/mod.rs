// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use self::inner::inner_get_share_sub_leader;
use super::PlacementCenterInterface;
use crate::poll::ClientPool;
use common_base::error::common::CommonError;
use inner::{
    inner_create_acl, inner_create_blacklist, inner_create_session, inner_create_topic,
    inner_create_user, inner_delete_acl, inner_delete_blacklist, inner_delete_session,
    inner_delete_topic, inner_delete_user, inner_list_acl, inner_list_blacklist,
    inner_list_session, inner_list_topic, inner_list_user, inner_save_last_will_message,
    inner_set_topic_retain_message, inner_update_session,
};
use mobc::Manager;
use protocol::placement_center::generate::mqtt::mqtt_service_client::MqttServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
mod inner;

async fn mqtt_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<MqttServiceClient<Channel>, CommonError> {
    match client_poll
        .placement_center_mqtt_services_client(addr)
        .await
    {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) async fn mqtt_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match mqtt_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::GetShareSub => {
                    inner_get_share_sub_leader(client, request.clone()).await
                }
                PlacementCenterInterface::ListUser => {
                    inner_list_user(client, request.clone()).await
                }
                PlacementCenterInterface::CreateUser => {
                    inner_create_user(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteUser => {
                    inner_delete_user(client, request.clone()).await
                }
                PlacementCenterInterface::ListTopic => {
                    inner_list_topic(client, request.clone()).await
                }
                PlacementCenterInterface::CreateTopic => {
                    inner_create_topic(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteTopic => {
                    inner_delete_topic(client, request.clone()).await
                }
                PlacementCenterInterface::SetTopicRetainMessage => {
                    inner_set_topic_retain_message(client, request.clone()).await
                }
                PlacementCenterInterface::ListSession => {
                    inner_list_session(client, request.clone()).await
                }
                PlacementCenterInterface::CreateSession => {
                    inner_create_session(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteSession => {
                    inner_delete_session(client, request.clone()).await
                }
                PlacementCenterInterface::UpdateSession => {
                    inner_update_session(client, request.clone()).await
                }
                PlacementCenterInterface::SaveLastWillMessage => {
                    inner_save_last_will_message(client, request.clone()).await
                }
                PlacementCenterInterface::ListAcl => inner_list_acl(client, request.clone()).await,

                PlacementCenterInterface::CreateAcl => {
                    inner_create_acl(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteAcl => {
                    inner_delete_acl(client, request.clone()).await
                }
                PlacementCenterInterface::ListBlackList => {
                    inner_list_blacklist(client, request.clone()).await
                }

                PlacementCenterInterface::CreateBlackList => {
                    inner_create_blacklist(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteBlackList => {
                    inner_delete_blacklist(client, request.clone()).await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "mqtt service does not support service interfaces [{:?}]",
                        interface
                    )))
                }
            };
            match result {
                Ok(data) => return Ok(data),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

#[derive(Clone)]
pub(crate) struct MQTTServiceManager {
    pub addr: String,
}

impl MQTTServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MQTTServiceManager {
    type Connection = MqttServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
                    "{},{}",
                    err.to_string(),
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
