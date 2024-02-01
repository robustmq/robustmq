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
use protocol::robust::meta::{
    meta_service_client::MetaServiceClient, SendRaftConfChangeReply, SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest, SetReply, SetRequest
};

pub async fn grpc_client_set(
    addr: &String,
    request: SetRequest,
) -> Result<SetReply, MetaError> {
    let mut client = match MetaServiceClient::connect(format!("http://{}", addr)).await {
        Ok(client) => client,
        Err(err) => return Err(MetaError::TonicTransport(err)),
    };

    let resp = match client.set(tonic::Request::new(request)).await {
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
