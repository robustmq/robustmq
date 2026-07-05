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

use kafka_protocol::error::ResponseError;
use tokio::sync::oneshot;

use crate::core::group_meta::{GroupMeta, GroupState};
use crate::core::sync::{sync_error, SyncResult};

pub struct LeaveOutcome {
    pub results: Vec<(String, i16)>,
    pub sync_wakeups: Vec<(oneshot::Sender<SyncResult>, SyncResult)>,
}

pub(crate) fn leave(group: &mut GroupMeta, member_ids: &[String]) -> LeaveOutcome {
    let mut results = Vec::with_capacity(member_ids.len());
    let mut removed = false;
    for id in member_ids {
        if group.members.remove(id).is_some() {
            removed = true;
            results.push((id.clone(), 0));
        } else {
            results.push((id.clone(), ResponseError::UnknownMemberId.code()));
        }
    }

    let mut sync_wakeups = Vec::new();
    if removed {
        if let Some(leader) = &group.leader_id {
            if !group.members.contains_key(leader) {
                group.leader_id = None;
            }
        }
        for member in group.members.values_mut() {
            if let Some(tx) = member.sync_waiter.take() {
                sync_wakeups.push((tx, sync_error(ResponseError::RebalanceInProgress.code())));
            }
        }
        if group.members.is_empty() {
            group.state = GroupState::Empty;
        } else {
            group.state = GroupState::PreparingRebalance;
        }
    }

    LeaveOutcome {
        results,
        sync_wakeups,
    }
}
