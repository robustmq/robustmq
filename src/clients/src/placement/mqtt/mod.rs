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

use super::PlacementCenterInterface;
use crate::poll::ClientPool;
use common_base::error::common::CommonError;
use mobc::{Connection, Manager};
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_client::MqttServiceClient, CreateAclRequest, CreateBlacklistRequest,
        CreateSessionRequest, CreateTopicRequest, CreateUserRequest, DeleteAclRequest,
        DeleteBlacklistRequest, DeleteSessionRequest, DeleteTopicRequest, DeleteUserRequest,
        GetShareSubLeaderReply, GetShareSubLeaderRequest, ListAclReply, ListAclRequest,
        ListBlacklistReply, ListBlacklistRequest, ListSessionReply, ListSessionRequest,
        ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
        SaveLastWillMessageRequest, SetTopicRetainMessageRequest, UpdateSessionRequest,
    },
};
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
mod inner;

async fn mqtt_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MQTTServiceManager>, CommonError> {
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
            let result =
                match interface {
                    PlacementCenterInterface::GetShareSubLeader => {
                        client_call(
                            client,
                            request.clone(),
                            |data| GetShareSubLeaderRequest::decode(data),
                            |mut client, request| async move {
                                client.get_share_sub_leader(request).await
                            },
                            |reply| GetShareSubLeaderReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::ListUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListUserRequest::decode(data),
                            |mut client, request| async move { client.list_user(request).await },
                            |reply| ListUserReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateUserRequest::decode(data),
                            |mut client, request| async move { client.create_user(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteUserRequest::decode(data),
                            |mut client, request| async move { client.delete_user(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::ListTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListTopicRequest::decode(data),
                            |mut client, request| async move { client.list_topic(request).await },
                            |reply| ListTopicReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateTopicRequest::decode(data),
                            |mut client, request| async move { client.create_topic(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteTopicRequest::decode(data),
                            |mut client: Connection<MQTTServiceManager>, request| async move {
                                client.delete_topic(request).await
                            },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::SetTopicRetainMessage => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SetTopicRetainMessageRequest::decode(data),
                            |mut client, request| async move {
                                client.set_topic_retain_message(request).await
                            },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::ListSession => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListSessionRequest::decode(data),
                            |mut client, request| async move { client.list_session(request).await },
                            |reply| ListSessionReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateSession => {
                            client_call(
                            client,
                            request.clone(),
                            |data| CreateSessionRequest::decode(data),
                            |mut client, request| async move { client.create_session(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteSession => {
                            client_call(
                            client,
                            request.clone(),
                            |data| DeleteSessionRequest::decode(data),
                            |mut client, request| async move { client.delete_session(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::UpdateSession => {
                            client_call(
                            client,
                            request.clone(),
                            |data| UpdateSessionRequest::decode(data),
                            |mut client, request| async move { client.update_session(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::SaveLastWillMessage => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SaveLastWillMessageRequest::decode(data),
                            |mut client, request| async move {
                                client.save_last_will_message(request).await
                            },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::ListAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListAclRequest::decode(data),
                            |mut client, request| async move { client.list_acl(request).await },
                            |reply| ListAclReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateAclRequest::decode(data),
                            |mut client, request| async move { client.create_acl(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteAclRequest::decode(data),
                            |mut client, request| async move { client.delete_acl(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::ListBlackList => {
                            client_call(
                            client,
                            request.clone(),
                            |data| ListBlacklistRequest::decode(data),
                            |mut client, request| async move { client.list_blacklist(request).await },
                            |reply| ListBlacklistReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateBlackList => {
                            client_call(
                            client,
                            request.clone(),
                            |data| CreateBlacklistRequest::decode(data),
                            |mut client, request| async move { client.create_blacklist(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteBlackList => {
                            client_call(
                            client,
                            request.clone(),
                            |data| DeleteBlacklistRequest::decode(data),
                            |mut client, request| async move { client.delete_blacklist(request).await },
                            |reply| CommonReply::encode_to_vec(reply),
                        )
                        .await
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
pub struct MQTTServiceManager {
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

pub(crate) async fn client_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
    client: Connection<MQTTServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<MQTTServiceManager>, R) -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<Resp>, tonic::Status>>,
    EncodeFunction: FnOnce(&Resp) -> Vec<u8>,
{
    match decode_fn(request.as_ref()) {
        Ok(decoded_request) => match client_fn(client, decoded_request).await {
            Ok(result) => {
                return Ok(encode_fn(&result.into_inner()));
            }
            Err(e) => return Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => return Err(CommonError::CommmonError(e.to_string())),
    }
}
