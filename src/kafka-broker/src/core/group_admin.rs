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

use bytes::Bytes;

use crate::core::group_meta::GroupMeta;

pub struct DescribedMemberInfo {
    pub member_id: String,
    pub group_instance_id: Option<String>,
    pub client_id: String,
    pub member_metadata: Bytes,
    pub member_assignment: Bytes,
}

pub struct DescribedGroupInfo {
    pub group_id: String,
    pub state: String,
    pub protocol_type: String,
    pub protocol_data: String,
    pub members: Vec<DescribedMemberInfo>,
}

pub struct ListedGroupInfo {
    pub group_id: String,
    pub protocol_type: String,
    pub state: String,
    pub group_type: String,
}

pub(crate) fn describe(group: &GroupMeta) -> DescribedGroupInfo {
    let selected = group.selected_protocol.as_deref();
    let mut members: Vec<DescribedMemberInfo> = group
        .members
        .values()
        .map(|m| DescribedMemberInfo {
            member_id: m.member_id.clone(),
            group_instance_id: m.group_instance_id.clone(),
            client_id: m.client_id.clone(),
            member_metadata: m.metadata_for(selected),
            member_assignment: m.assignment.clone(),
        })
        .collect();
    members.sort_by(|a, b| a.member_id.cmp(&b.member_id));

    DescribedGroupInfo {
        group_id: group.group_id.clone(),
        state: group.state.name().to_string(),
        protocol_type: group.protocol_type.clone().unwrap_or_default(),
        protocol_data: group.selected_protocol.clone().unwrap_or_default(),
        members,
    }
}
