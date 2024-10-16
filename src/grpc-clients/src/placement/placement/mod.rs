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
use protocol::placement_center::placement_center_inner::placement_center_service_client::PlacementCenterServiceClient;
use protocol::placement_center::placement_center_inner::{
    ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataReply,
    DeleteIdempotentDataRequest, DeleteResourceConfigReply, DeleteResourceConfigRequest,
    ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, NodeListReply, NodeListRequest,
    RegisterNodeReply, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest,
    SendRaftMessageReply, SendRaftMessageRequest, SetIdempotentDataReply, SetIdempotentDataRequest,
    SetResourceConfigReply, SetResourceConfigRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};
use tonic::transport::Channel;

use super::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;

pub(crate) async fn placement_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match placement_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result: Result<Vec<u8>, CommonError> = match interface {
                    PlacementCenterInterface::ClusterStatus => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ClusterStatusRequest::decode(data),
                            |mut client, request| async move { client.cluster_status(request).await },
                            ClusterStatusReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::ListNode => {
                        client_call(
                            client,
                            request.clone(),
                            |data| NodeListRequest::decode(data),
                            |mut client, request| async move { client.node_list(request).await },
                            NodeListReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::RegisterNode => {
                        client_call(
                            client,
                            request.clone(),
                            |data| RegisterNodeRequest::decode(data),
                            |mut client: Connection<PlacementServiceManager>, request| async move { client.register_node(request).await },
                            RegisterNodeReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::UnRegisterNode => {
                        client_call(
                            client,
                            request.clone(),
                            |data| UnRegisterNodeRequest::decode(data),
                            |mut client, request| async move { client.un_register_node(request).await },
                            UnRegisterNodeReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::Heartbeat => {
                        client_call(
                            client,
                            request.clone(),
                            |data| HeartbeatRequest::decode(data),
                            |mut client, request| async move { client.heartbeat(request).await },
                            HeartbeatReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::SendRaftMessage => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SendRaftMessageRequest::decode(data),
                            |mut client, request| async move { client.send_raft_message(request).await },
                            SendRaftMessageReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::SendRaftConfChange => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SendRaftConfChangeRequest::decode(data),
                            |mut client, request| async move { client.send_raft_conf_change(request).await },
                            SendRaftConfChangeReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::SetReourceConfig => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SetResourceConfigRequest::decode(data),
                            |mut client, request| async move { client.set_resource_config(request).await },
                            SetResourceConfigReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::GetReourceConfig => {
                        client_call(
                            client,
                            request.clone(),
                            |data| GetResourceConfigRequest::decode(data),
                            |mut client, request| async move { client.get_resource_config(request).await },
                            GetResourceConfigReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::DeleteReourceConfig => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteResourceConfigRequest::decode(data),
                            |mut client, request| async move { client.delete_resource_config(request).await },
                            DeleteResourceConfigReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::SetIdempotentData => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SetIdempotentDataRequest::decode(data),
                            |mut client, request| async move { client.set_idempotent_data(request).await },
                            SetIdempotentDataReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::ExistsIdempotentData => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ExistsIdempotentDataRequest::decode(data),
                            |mut client, request| async move { client.exists_idempotent_data(request).await },
                            ExistsIdempotentDataReply::encode_to_vec,
                        ).await
                    }
                    PlacementCenterInterface::DeleteIdempotentData => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteIdempotentDataRequest::decode(data),
                            |mut client, request| async move { client.delete_idempotent_data(request).await },
                            DeleteIdempotentDataReply::encode_to_vec,
                        ).await
                    }
                    _ => {
                        return Err(CommonError::CommmonError(format!(
                            "placement service does not support service interfaces [{:?}]",
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

async fn placement_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<PlacementServiceManager>, CommonError> {
    match client_poll
        .placement_center_inner_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub struct PlacementServiceManager {
    pub addr: String,
}

impl PlacementServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for PlacementServiceManager {
    type Connection = PlacementCenterServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match PlacementCenterServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
                    "manager connect error:{},{}",
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
    client: Connection<PlacementServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<PlacementServiceManager>, R) -> Fut,
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
