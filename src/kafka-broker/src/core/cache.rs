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

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use kafka_protocol::error::ResponseError;
use tokio::sync::oneshot;

use crate::core::assignor::TopicMeta;
use crate::core::consumer_group_meta::{self, ConsumerDescribedGroup, ConsumerGroupMeta};
use crate::core::consumer_heartbeat::{
    self, heartbeat_error, ConsumerHeartbeatParams, ConsumerHeartbeatResult,
};
use crate::core::group_admin::{self, DescribedGroupInfo, ListedGroupInfo};
use crate::core::group_meta::{GroupMeta, MemberMeta};
use crate::core::heartbeat;
use crate::core::join::{self, AddMemberOutcome, JoinCompletion};
use crate::core::leave::{self, LeaveOutcome};
use crate::core::sync::{self, sync_error, SyncOutcome, SyncResult};

// In-memory data the Kafka broker caches on the coordinator node (consumer-group
// state, keyed by group_id). Pure lock routing: each method takes the group's
// per-key lock and delegates to the phase logic in join/sync/leave/heartbeat.
#[derive(Default)]
pub struct KafkaCacheManager {
    // Classic-protocol groups.
    groups: DashMap<String, GroupMeta>,
    // KIP-848 consumer groups; a group id belongs to exactly one protocol.
    consumer_groups: DashMap<String, ConsumerGroupMeta>,
}

impl KafkaCacheManager {
    pub fn new() -> Self {
        KafkaCacheManager {
            groups: DashMap::with_capacity(8),
            consumer_groups: DashMap::with_capacity(8),
        }
    }

    pub fn has_consumer_group(&self, group_id: &str) -> bool {
        self.consumer_groups.contains_key(group_id)
    }

    pub fn add_member(&self, group_id: &str, member: MemberMeta) -> AddMemberOutcome {
        let mut group = self
            .groups
            .entry(group_id.to_string())
            .or_insert_with(|| GroupMeta::new(group_id.to_string()));
        join::register_member(&mut group, member)
    }

    pub fn finish_rebalance(&self, group_id: &str) -> JoinCompletion {
        self.groups
            .get_mut(group_id)
            .map(|mut group| join::complete_join(&mut group))
            .unwrap_or_default()
    }

    pub fn sync_member(
        &self,
        group_id: &str,
        member_id: &str,
        generation_id: i32,
        assignments: Vec<(String, bytes::Bytes)>,
    ) -> SyncOutcome {
        match self.groups.get_mut(group_id) {
            Some(mut group) => sync::sync(&mut group, member_id, generation_id, assignments),
            None => SyncOutcome::Immediate(sync_error(ResponseError::UnknownMemberId.code())),
        }
    }

    pub fn expire_sync(
        &self,
        group_id: &str,
        generation_id: i32,
    ) -> Vec<(oneshot::Sender<SyncResult>, SyncResult)> {
        self.groups
            .get_mut(group_id)
            .map(|mut group| sync::expire_sync(&mut group, generation_id))
            .unwrap_or_default()
    }

    pub fn heartbeat_member(&self, group_id: &str, member_id: &str, generation_id: i32) -> i16 {
        match self.groups.get_mut(group_id) {
            Some(mut group) => heartbeat::heartbeat(&mut group, member_id, generation_id),
            None => ResponseError::UnknownMemberId.code(),
        }
    }

    pub fn reap_expired_members(&self, now_ms: u128, consumer_session_timeout_ms: u64) {
        for mut group in self.groups.iter_mut() {
            heartbeat::remove_expired_members(&mut group, now_ms);
        }
        for mut group in self.consumer_groups.iter_mut() {
            consumer_heartbeat::remove_expired_members(
                &mut group,
                now_ms,
                consumer_session_timeout_ms,
            );
        }
    }

    pub fn remove_members(&self, group_id: &str, member_ids: &[String]) -> LeaveOutcome {
        match self.groups.get_mut(group_id) {
            Some(mut group) => leave::leave(&mut group, member_ids),
            None => LeaveOutcome {
                results: member_ids
                    .iter()
                    .map(|id| (id.clone(), ResponseError::UnknownMemberId.code()))
                    .collect(),
                sync_wakeups: Vec::new(),
            },
        }
    }

    pub fn describe_group(&self, group_id: &str) -> Option<DescribedGroupInfo> {
        self.groups
            .get(group_id)
            .map(|group| group_admin::describe(&group))
    }

    pub fn list_groups(&self) -> Vec<ListedGroupInfo> {
        let mut groups: Vec<ListedGroupInfo> = self
            .groups
            .iter()
            .map(|group| ListedGroupInfo {
                group_id: group.group_id.clone(),
                protocol_type: group.protocol_type.clone().unwrap_or_default(),
                state: group.state.name().to_string(),
                group_type: "classic".to_string(),
            })
            .collect();
        groups.extend(self.consumer_groups.iter().map(|group| ListedGroupInfo {
            group_id: group.group_id.clone(),
            protocol_type: "consumer".to_string(),
            state: group.state_name().to_string(),
            group_type: "consumer".to_string(),
        }));
        groups.sort_by(|a, b| a.group_id.cmp(&b.group_id));
        groups
    }

    pub fn consumer_heartbeat(
        &self,
        params: ConsumerHeartbeatParams,
        resolve_topic: &dyn Fn(&str) -> Option<TopicMeta>,
        now_ms: u128,
    ) -> ConsumerHeartbeatResult {
        if self.groups.contains_key(&params.group_id) {
            return heartbeat_error(
                ResponseError::GroupIdNotFound.code(),
                "group id is used by a classic-protocol group",
                &params.member_id,
            );
        }
        let group_id = params.group_id.clone();
        let mut group = self
            .consumer_groups
            .entry(group_id.clone())
            .or_insert_with(|| ConsumerGroupMeta::new(group_id));
        consumer_heartbeat::heartbeat(&mut group, params, resolve_topic, now_ms)
    }

    pub fn describe_consumer_group(
        &self,
        group_id: &str,
        tenant: &str,
    ) -> Option<ConsumerDescribedGroup> {
        self.consumer_groups
            .get(group_id)
            .map(|group| consumer_group_meta::describe(&group, tenant))
    }

    pub fn delete_group(&self, group_id: &str) -> i16 {
        match self.groups.entry(group_id.to_string()) {
            Entry::Occupied(entry) => {
                return if entry.get().members.is_empty() {
                    entry.remove();
                    0
                } else {
                    ResponseError::NonEmptyGroup.code()
                };
            }
            Entry::Vacant(_) => {}
        }
        match self.consumer_groups.entry(group_id.to_string()) {
            Entry::Occupied(entry) => {
                if entry.get().members.is_empty() {
                    entry.remove();
                    0
                } else {
                    ResponseError::NonEmptyGroup.code()
                }
            }
            Entry::Vacant(_) => ResponseError::GroupIdNotFound.code(),
        }
    }
}
