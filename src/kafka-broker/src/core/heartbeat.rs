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
use std::time::Duration;

use common_base::tools::now_millis;
use kafka_protocol::error::ResponseError;

use crate::core::cache::KafkaCacheManager;
use crate::core::group_meta::{GroupMeta, GroupState};

pub fn heartbeat(group: &mut GroupMeta, member_id: &str, generation_id: i32) -> i16 {
    let Some(member) = group.members.get_mut(member_id) else {
        return ResponseError::UnknownMemberId.code();
    };
    if generation_id != group.generation_id {
        return ResponseError::IllegalGeneration.code();
    }

    member.last_heartbeat_ms = now_millis();

    match group.state {
        GroupState::Stable => 0,
        GroupState::PreparingRebalance | GroupState::CompletingRebalance => {
            ResponseError::RebalanceInProgress.code()
        }
        GroupState::Empty | GroupState::Dead => ResponseError::UnknownMemberId.code(),
    }
}

pub fn remove_expired_members(group: &mut GroupMeta, now_ms: u128) {
    if group.state != GroupState::Stable {
        return;
    }
    let before = group.members.len();
    group.members.retain(|_, m| {
        now_ms.saturating_sub(m.last_heartbeat_ms) <= m.session_timeout_ms.max(0) as u128
    });
    if group.members.len() == before {
        return;
    }

    if let Some(leader) = &group.leader_id {
        if !group.members.contains_key(leader) {
            group.leader_id = None;
        }
    }
    if group.members.is_empty() {
        group.state = GroupState::Empty;
    } else {
        group.state = GroupState::PreparingRebalance;
    }
}

pub fn spawn_session_reaper(
    cache: Arc<KafkaCacheManager>,
    interval_ms: u64,
    consumer_session_timeout_ms: u64,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            cache.reap_expired_members(now_millis(), consumer_session_timeout_ms);
        }
    });
}
