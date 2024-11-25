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

use std::sync::Arc;

use common_base::error::common::CommonError;
use mobc::{Connection, Manager};
use prost::Message;
use protocol::placement_center::placement_center_mqtt::mqtt_service_client::MqttServiceClient;
use protocol::placement_center::placement_center_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateUserReply, CreateUserRequest, DeleteAclRequest, DeleteAclReply,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteSessionReply, DeleteSessionRequest,
    DeleteTopicReply, DeleteTopicRequest, DeleteUserReply, DeleteUserRequest,
    GetShareSubLeaderReply, GetShareSubLeaderRequest, ListAclReply, ListAclRequest,
    ListBlacklistReply, ListBlacklistRequest, ListSessionReply, ListSessionRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest, SaveLastWillMessageReply,
    SaveLastWillMessageRequest, SetTopicRetainMessageReply, SetTopicRetainMessageRequest,
    UpdateSessionReply, UpdateSessionRequest,
};
use tonic::transport::Channel;

use super::PlacementCenterInterface;
use crate::pool::ClientPool;

pub mod call;

async fn mqtt_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttServiceManager>, CommonError> {
    match client_pool
        .placement_center_mqtt_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn mqtt_interface_call(
    interface: PlacementCenterInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match mqtt_client(client_pool.clone(), addr.clone()).await {
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
                            GetShareSubLeaderReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::ListUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListUserRequest::decode(data),
                            |mut client, request| async move { client.list_user(request).await },
                            ListUserReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateUserRequest::decode(data),
                            |mut client, request| async move { client.create_user(request).await },
                            CreateUserReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteUserRequest::decode(data),
                            |mut client, request| async move { client.delete_user(request).await },
                            DeleteUserReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::ListTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListTopicRequest::decode(data),
                            |mut client, request| async move { client.list_topic(request).await },
                            ListTopicReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateTopicRequest::decode(data),
                            |mut client, request| async move { client.create_topic(request).await },
                            CreateTopicReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteTopic => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteTopicRequest::decode(data),
                            |mut client: Connection<MqttServiceManager>, request| async move {
                                client.delete_topic(request).await
                            },
                            DeleteTopicReply::encode_to_vec,
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
                            SetTopicRetainMessageReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::ListSession => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListSessionRequest::decode(data),
                            |mut client, request| async move { client.list_session(request).await },
                            ListSessionReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateSession => client_call(
                        client,
                        request.clone(),
                        |data| CreateSessionRequest::decode(data),
                        |mut client, request| async move { client.create_session(request).await },
                        CreateSessionReply::encode_to_vec,
                    )
                    .await,
                    PlacementCenterInterface::DeleteSession => client_call(
                        client,
                        request.clone(),
                        |data| DeleteSessionRequest::decode(data),
                        |mut client, request| async move { client.delete_session(request).await },
                        DeleteSessionReply::encode_to_vec,
                    )
                    .await,
                    PlacementCenterInterface::UpdateSession => client_call(
                        client,
                        request.clone(),
                        |data| UpdateSessionRequest::decode(data),
                        |mut client, request| async move { client.update_session(request).await },
                        UpdateSessionReply::encode_to_vec,
                    )
                    .await,
                    PlacementCenterInterface::SaveLastWillMessage => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SaveLastWillMessageRequest::decode(data),
                            |mut client, request| async move {
                                client.save_last_will_message(request).await
                            },
                            SaveLastWillMessageReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::ListAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListAclRequest::decode(data),
                            |mut client, request| async move { client.list_acl(request).await },
                            ListAclReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::CreateAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateAclRequest::decode(data),
                            |mut client, request| async move { client.create_acl(request).await },
                            CreateAclReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::DeleteAcl => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteAclRequest::decode(data),
                            |mut client, request| async move { client.delete_acl(request).await },
                            DeleteAclReply::encode_to_vec,
                        )
                        .await
                    }
                    PlacementCenterInterface::ListBlackList => client_call(
                        client,
                        request.clone(),
                        |data| ListBlacklistRequest::decode(data),
                        |mut client, request| async move { client.list_blacklist(request).await },
                        ListBlacklistReply::encode_to_vec,
                    )
                    .await,
                    PlacementCenterInterface::CreateBlackList => client_call(
                        client,
                        request.clone(),
                        |data| CreateBlacklistRequest::decode(data),
                        |mut client, request| async move { client.create_blacklist(request).await },
                        CreateBlacklistReply::encode_to_vec,
                    )
                    .await,
                    PlacementCenterInterface::DeleteBlackList => client_call(
                        client,
                        request.clone(),
                        |data| DeleteBlacklistRequest::decode(data),
                        |mut client, request| async move { client.delete_blacklist(request).await },
                        DeleteBlacklistReply::encode_to_vec,
                    )
                    .await,
                    _ => {
                        return Err(CommonError::CommmonError(format!(
                            "mqtt service does not support service interfaces [{:?}]",
                            interface
                        )))
                    }
                };
            match result {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct MqttServiceManager {
    pub addr: String,
}

impl MqttServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MqttServiceManager {
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
                    err,
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
    client: Connection<MqttServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<MqttServiceManager>, R) -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<Resp>, tonic::Status>>,
    EncodeFunction: FnOnce(&Resp) -> Vec<u8>,
{
    match decode_fn(request.as_ref()) {
        Ok(decoded_request) => match client_fn(client, decoded_request).await {
            Ok(result) => Ok(encode_fn(&result.into_inner())),
            Err(e) => Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => Err(CommonError::CommmonError(e.to_string())),
    }
}
