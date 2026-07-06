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

use std::collections::HashMap;

use bytes::Bytes;
use tokio::sync::oneshot;

use crate::core::join::JoinResult;
use crate::core::sync::SyncResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupState {
    Empty,
    PreparingRebalance,
    CompletingRebalance,
    Stable,
    Dead,
}

impl GroupState {
    pub fn name(&self) -> &'static str {
        match self {
            GroupState::Empty => "Empty",
            GroupState::PreparingRebalance => "PreparingRebalance",
            GroupState::CompletingRebalance => "CompletingRebalance",
            GroupState::Stable => "Stable",
            GroupState::Dead => "Dead",
        }
    }
}

pub struct GroupMeta {
    pub group_id: String,
    pub state: GroupState,
    pub generation_id: i32,
    pub protocol_type: Option<String>,
    pub selected_protocol: Option<String>,
    pub leader_id: Option<String>,
    pub members: HashMap<String, MemberMeta>,
    pub rebalance_timer_running: bool,
}

impl GroupMeta {
    pub fn new(group_id: String) -> Self {
        GroupMeta {
            group_id,
            state: GroupState::Empty,
            generation_id: 0,
            protocol_type: None,
            selected_protocol: None,
            leader_id: None,
            members: HashMap::new(),
            rebalance_timer_running: false,
        }
    }
}

pub struct MemberMeta {
    pub member_id: String,
    pub group_instance_id: Option<String>,
    pub client_id: String,
    pub session_timeout_ms: i32,
    pub rebalance_timeout_ms: i32,
    pub protocol_type: String,
    pub protocols: Vec<(String, Bytes)>,
    pub assignment: Bytes,
    pub join_waiter: Option<oneshot::Sender<JoinResult>>,
    pub sync_waiter: Option<oneshot::Sender<SyncResult>>,
    pub last_heartbeat_ms: u128,
}

impl MemberMeta {
    pub(crate) fn metadata_for(&self, protocol: Option<&str>) -> Bytes {
        self.protocols
            .iter()
            .find(|(n, _)| Some(n.as_str()) == protocol)
            .map(|(_, b)| b.clone())
            .unwrap_or_default()
    }
}
