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
use protocol::placement_center::placement_center_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateUserReply, CreateUserRequest, DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply,
    DeleteBlacklistRequest, DeleteExclusiveTopicReply, DeleteExclusiveTopicRequest,
    DeleteSessionReply, DeleteSessionRequest, DeleteTopicReply, DeleteTopicRequest,
    DeleteUserReply, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListSessionReply,
    ListSessionRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetExclusiveTopicReply,
    SetExclusiveTopicRequest, SetTopicRetainMessageReply, SetTopicRetainMessageRequest,
    UpdateSessionReply, UpdateSessionRequest,
};

use super::{MqttServiceReply, MqttServiceRequest};
use crate::placement::{retry_placement_center_call, PlacementCenterReply, PlacementCenterRequest};
use crate::pool::ClientPool;

macro_rules! generate_mqtt_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: Arc<ClientPool>,
            addrs: &[String],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            let request = PlacementCenterRequest::Mqtt(MqttServiceRequest::$variant(request));
            match retry_placement_center_call(&client_pool, addrs, request).await? {
                PlacementCenterReply::Mqtt(MqttServiceReply::$variant(reply)) => Ok(reply),
                _ => unreachable!("Reply type mismatch"),
            }
        }
    };
}

generate_mqtt_service_call!(
    placement_get_share_sub_leader,
    GetShareSubLeaderRequest,
    GetShareSubLeaderReply,
    GetShareSubLeader
);
generate_mqtt_service_call!(
    placement_create_user,
    CreateUserRequest,
    CreateUserReply,
    CreateUser
);
generate_mqtt_service_call!(
    placement_delete_user,
    DeleteUserRequest,
    DeleteUserReply,
    DeleteUser
);
generate_mqtt_service_call!(
    placement_list_user,
    ListUserRequest,
    ListUserReply,
    ListUser
);
generate_mqtt_service_call!(
    placement_create_topic,
    CreateTopicRequest,
    CreateTopicReply,
    CreateTopic
);
generate_mqtt_service_call!(
    placement_delete_topic,
    DeleteTopicRequest,
    DeleteTopicReply,
    DeleteTopic
);
generate_mqtt_service_call!(
    placement_list_topic,
    ListTopicRequest,
    ListTopicReply,
    ListTopic
);
generate_mqtt_service_call!(
    placement_set_topic_retain_message,
    SetTopicRetainMessageRequest,
    SetTopicRetainMessageReply,
    SetTopicRetainMessage
);
generate_mqtt_service_call!(
    placement_set_nx_exclusive_topic,
    SetExclusiveTopicRequest,
    SetExclusiveTopicReply,
    SetNxExclusiveTopic
);
generate_mqtt_service_call!(
    placement_delete_exclusive_topic,
    DeleteExclusiveTopicRequest,
    DeleteExclusiveTopicReply,
    DeleteExclusiveTopic
);
generate_mqtt_service_call!(
    placement_create_session,
    CreateSessionRequest,
    CreateSessionReply,
    CreateSession
);
generate_mqtt_service_call!(
    placement_delete_session,
    DeleteSessionRequest,
    DeleteSessionReply,
    DeleteSession
);
generate_mqtt_service_call!(
    placement_list_session,
    ListSessionRequest,
    ListSessionReply,
    ListSession
);
generate_mqtt_service_call!(
    placement_update_session,
    UpdateSessionRequest,
    UpdateSessionReply,
    UpdateSession
);
generate_mqtt_service_call!(
    placement_save_last_will_message,
    SaveLastWillMessageRequest,
    SaveLastWillMessageReply,
    SaveLastWillMessage
);
generate_mqtt_service_call!(create_acl, CreateAclRequest, CreateAclReply, CreateAcl);
generate_mqtt_service_call!(delete_acl, DeleteAclRequest, DeleteAclReply, DeleteAcl);
generate_mqtt_service_call!(list_acl, ListAclRequest, ListAclReply, ListAcl);
generate_mqtt_service_call!(
    create_blacklist,
    CreateBlacklistRequest,
    CreateBlacklistReply,
    CreateBlacklist
);
generate_mqtt_service_call!(
    list_blacklist,
    ListBlacklistRequest,
    ListBlacklistReply,
    ListBlacklist
);
generate_mqtt_service_call!(
    delete_blacklist,
    DeleteBlacklistRequest,
    DeleteBlacklistReply,
    DeleteBlacklist
);
