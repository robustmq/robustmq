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
use prost::Message;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
};

use crate::placement::{retry_call, PlacementCenterInterface, PlacementCenterService};
use crate::poll::ClientPool;

pub async fn create_shard(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateShardRequest,
) -> Result<CreateShardReply, CommonError> {
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
        Ok(data) => match CreateShardReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_shard(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteShardRequest,
) -> Result<DeleteShardReply, CommonError> {
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
        Ok(data) => match DeleteShardReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn create_next_segment(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateNextSegmentRequest,
) -> Result<CreateNextSegmentReply, CommonError> {
    let request_data = CreateNextSegmentRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Journal,
        PlacementCenterInterface::CreateSegment,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CreateNextSegmentReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_segment(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, CommonError> {
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
        Ok(data) => match DeleteSegmentReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
