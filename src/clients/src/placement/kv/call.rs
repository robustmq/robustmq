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


use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService},
    poll::ClientPool,
};
use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest},
};
use std::sync::Arc;

pub async fn placement_set(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SetRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Set,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_get(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetRequest,
) -> Result<GetReply, CommonError> {
    let request_data = GetRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Get,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Delete,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_exists(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ExistsRequest,
) -> Result<ExistsReply, CommonError> {
    let request_data = ExistsRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Kv,
        PlacementCenterInterface::Exists,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ExistsReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
