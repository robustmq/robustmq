// Copyright 2023 RobustMQ Team
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


use crate::placement::PlacementCenterService;
use crate::placement::{retry_call, PlacementCenterInterface};
use crate::poll::ClientPool;
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::journal::{
    CreateSegmentRequest, DeleteSegmentRequest, DeleteShardRequest,
};
use protocol::placement_center::generate::{common::CommonReply, journal::CreateShardRequest};
use std::sync::Arc;

pub async fn create_shard(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateShardRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateShardRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::CreateShard,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn delete_shard(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteShardRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteShardRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::DeleteShard,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn create_segment(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateSegmentRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::CreateSegment,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn delete_segment(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteSegmentRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::DeleteSegment,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
