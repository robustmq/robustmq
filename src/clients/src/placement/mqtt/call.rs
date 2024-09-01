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
    mqtt::{
        CreateAclRequest, CreateBlacklistRequest, CreateSessionRequest, CreateTopicRequest,
        CreateUserRequest, DeleteAclRequest, DeleteBlacklistRequest, DeleteSessionRequest,
        DeleteTopicRequest, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
        ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListSessionReply,
        ListSessionRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
        SaveLastWillMessageRequest, SetTopicRetainMessageRequest, UpdateSessionRequest,
    },
};
use std::sync::Arc;

pub async fn placement_get_share_sub_leader(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, CommonError> {
    let request_data = GetShareSubLeaderRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::GetShareSub,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetShareSubLeaderReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_create_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateUserRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = CreateUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateUser,
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

pub async fn placement_delete_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteUserRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteUser,
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

pub async fn placement_list_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListUserRequest,
) -> Result<ListUserReply, CommonError> {
    let request_data = ListUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListUser,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListUserReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
pub async fn placement_create_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateTopicRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = CreateTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateTopic,
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

pub async fn placement_delete_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteTopicRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteTopic,
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

pub async fn placement_list_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListTopicRequest,
) -> Result<ListTopicReply, CommonError> {
    let request_data = ListTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListTopic,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListTopicReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_set_topic_retain_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetTopicRetainMessageRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SetTopicRetainMessageRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::SetTopicRetainMessage,
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

pub async fn placement_create_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateSessionRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = CreateSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateSession,
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

pub async fn placement_delete_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteSession,
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

pub async fn placement_list_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListSessionRequest,
) -> Result<ListSessionReply, CommonError> {
    let request_data = ListSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListSessionReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_update_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateSessionRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = UpdateSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::UpdateSession,
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

pub async fn placement_save_last_will_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SaveLastWillMessageRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SaveLastWillMessageRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::SaveLastWillMessage,
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

pub async fn list_acl(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListAclRequest,
) -> Result<ListAclReply, CommonError> {
    let request_data = ListAclRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListAcl,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListAclReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn create_acl(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateAclRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = CreateAclRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateAcl,
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

pub async fn delete_acl(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteAclRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteAclRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteAcl,
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

pub async fn list_blacklist(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListBlacklistRequest,
) -> Result<ListBlacklistReply, CommonError> {
    let request_data = ListBlacklistRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListBlackList,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListBlacklistReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn create_blacklist(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateBlacklistRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = CreateBlacklistRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateBlackList,
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

pub async fn delete_blacklist(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteBlacklistRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteBlacklistRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteBlackList,
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
