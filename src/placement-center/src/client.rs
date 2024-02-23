/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::errors::MetaError;
use protocol::placement_center::placement::{
    meta_service_client::MetaServiceClient, CommonReply, CreateShardRequest, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest, UnRegisterNodeRequest
};


pub async fn register_node(
    addr: &String,
    request: RegisterNodeRequest,
) -> Result<CommonReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };

    let resp = match client.register_node(tonic::Request::new(request)).await {
        Ok(reply) => reply.into_inner(),
        Err(status) => return Err(MetaError::MetaGrpcStatus(status)),
    };
    return Ok(resp);
}

pub async fn unregister_node(
    addr: &String,
    request: UnRegisterNodeRequest,
) -> Result<CommonReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };

    let resp = match client.un_register_node(tonic::Request::new(request)).await {
        Ok(reply) => reply.into_inner(),
        Err(status) => return Err(MetaError::MetaGrpcStatus(status)),
    };
    return Ok(resp);
}

pub async fn create_shard(
    addr: &String,
    request: CreateShardRequest,
) -> Result<CommonReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };

    let resp = match client.create_shard(tonic::Request::new(request)).await {
        Ok(reply) => reply.into_inner(),
        Err(status) => return Err(MetaError::MetaGrpcStatus(status)),
    };
    return Ok(resp);
}

pub async fn send_raft_message(
    addr: &String,
    message: Vec<u8>,
) -> Result<SendRaftMessageReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };
    let request = tonic::Request::new(SendRaftMessageRequest { message });

    let resp = match client.send_raft_message(request).await {
        Ok(reply) => reply.into_inner(),
        Err(status) => return Err(MetaError::MetaGrpcStatus(status)),
    };
    return Ok(resp);
}

pub async fn send_raft_conf_change(
    addr: &String,
    message: Vec<u8>,
) -> Result<SendRaftConfChangeReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };
    let request = tonic::Request::new(SendRaftConfChangeRequest { message });

    let resp = match client.send_raft_conf_change(request).await {
        Ok(reply) => reply.into_inner(),
        Err(status) => return Err(MetaError::MetaGrpcStatus(status)),
    };
    return Ok(resp);
}
