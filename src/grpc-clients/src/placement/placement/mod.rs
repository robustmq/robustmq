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
use crate::pool::ClientPool;

pub mod call;

#[derive(Debug, Clone)]
pub enum PlacementServiceRequest {
    ClusterStatus(ClusterStatusRequest),
    ListNode(NodeListRequest),
    RegisterNode(RegisterNodeRequest),
    UnRegisterNode(UnRegisterNodeRequest),
    Heartbeat(HeartbeatRequest),
    SendRaftMessage(SendRaftMessageRequest),
    SendRaftConfChange(SendRaftConfChangeRequest),
    SetResourceConfig(SetResourceConfigRequest),
    GetResourceConfig(GetResourceConfigRequest),
    DeleteResourceConfig(DeleteResourceConfigRequest),
    SetIdempotentData(SetIdempotentDataRequest),
    ExistsIdempotentData(ExistsIdempotentDataRequest),
    DeleteIdempotentData(DeleteIdempotentDataRequest),
}

#[derive(Debug, Clone)]
pub enum PlacementServiceReply {
    ClusterStatus(ClusterStatusReply),
    ListNode(NodeListReply),
    RegisterNode(RegisterNodeReply),
    UnRegisterNode(UnRegisterNodeReply),
    Heartbeat(HeartbeatReply),
    SendRaftMessage(SendRaftMessageReply),
    SendRaftConfChange(SendRaftConfChangeReply),
    SetResourceConfig(SetResourceConfigReply),
    GetResourceConfig(GetResourceConfigReply),
    DeleteResourceConfig(DeleteResourceConfigReply),
    SetIdempotentData(SetIdempotentDataReply),
    ExistsIdempotentData(ExistsIdempotentDataReply),
    DeleteIdempotentData(DeleteIdempotentDataReply),
}

pub(super) async fn call_placement_service_once(
    client_pool: &ClientPool,
    addr: &str,
    request: PlacementServiceRequest,
) -> Result<PlacementServiceReply, CommonError> {
    use PlacementServiceRequest::*;

    match request {
        ClusterStatus(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.cluster_status(request).await?;
            Ok(PlacementServiceReply::ClusterStatus(reply.into_inner()))
        }
        ListNode(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.node_list(request).await?;
            Ok(PlacementServiceReply::ListNode(reply.into_inner()))
        }
        RegisterNode(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.register_node(request).await?;
            Ok(PlacementServiceReply::RegisterNode(reply.into_inner()))
        }
        UnRegisterNode(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.un_register_node(request).await?;
            Ok(PlacementServiceReply::UnRegisterNode(reply.into_inner()))
        }
        Heartbeat(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.heartbeat(request).await?;
            Ok(PlacementServiceReply::Heartbeat(reply.into_inner()))
        }
        SendRaftMessage(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.send_raft_message(request).await?;
            Ok(PlacementServiceReply::SendRaftMessage(reply.into_inner()))
        }
        SendRaftConfChange(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.send_raft_conf_change(request).await?;
            Ok(PlacementServiceReply::SendRaftConfChange(reply.into_inner()))
        }
        SetResourceConfig(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.set_resource_config(request).await?;
            Ok(PlacementServiceReply::SetResourceConfig(reply.into_inner()))
        }
        GetResourceConfig(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.get_resource_config(request).await?;
            Ok(PlacementServiceReply::GetResourceConfig(reply.into_inner()))
        }
        DeleteResourceConfig(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.delete_resource_config(request).await?;
            Ok(PlacementServiceReply::DeleteResourceConfig(reply.into_inner()))
        }
        SetIdempotentData(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.set_idempotent_data(request).await?;
            Ok(PlacementServiceReply::SetIdempotentData(reply.into_inner()))
        }
        ExistsIdempotentData(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.exists_idempotent_data(request).await?;
            Ok(PlacementServiceReply::ExistsIdempotentData(reply.into_inner()))
        }
        DeleteIdempotentData(request) => {
            let mut client = client_pool.placement_center_inner_services_client(addr).await?;
            let reply = client.delete_idempotent_data(request).await?;
            Ok(PlacementServiceReply::DeleteIdempotentData(reply.into_inner()))
        }
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
                return Err(CommonError::CommonError(format!(
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
        Err(e) => Err(CommonError::CommonError(e.to_string())),
    }
}
