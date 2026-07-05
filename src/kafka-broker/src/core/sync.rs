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
use kafka_protocol::error::ResponseError;
use tokio::sync::oneshot;

use crate::core::group_meta::{GroupMeta, GroupState};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub error_code: i16,
    pub protocol_type: Option<String>,
    pub protocol_name: Option<String>,
    pub assignment: Bytes,
}

pub enum SyncOutcome {
    Immediate(SyncResult),
    Park(oneshot::Receiver<SyncResult>),
    Completed {
        own: SyncResult,
        wakeups: Vec<(oneshot::Sender<SyncResult>, SyncResult)>,
    },
}

pub struct SyncGroupParams {
    pub group_id: String,
    pub member_id: String,
    pub generation_id: i32,
    pub assignments: Vec<(String, Bytes)>,
}

pub(crate) fn sync_error(error_code: i16) -> SyncResult {
    SyncResult {
        error_code,
        protocol_type: None,
        protocol_name: None,
        assignment: Bytes::new(),
    }
}

pub(crate) fn sync(
    group: &mut GroupMeta,
    member_id: &str,
    generation_id: i32,
    assignments: Vec<(String, Bytes)>,
) -> SyncOutcome {
    if !group.members.contains_key(member_id) {
        return SyncOutcome::Immediate(sync_error(ResponseError::UnknownMemberId.code()));
    }
    if generation_id != group.generation_id {
        return SyncOutcome::Immediate(sync_error(ResponseError::IllegalGeneration.code()));
    }

    match group.state {
        GroupState::Stable => {
            let assignment = group.members[member_id].assignment.clone();
            SyncOutcome::Immediate(SyncResult {
                error_code: 0,
                protocol_type: group.protocol_type.clone(),
                protocol_name: group.selected_protocol.clone(),
                assignment,
            })
        }
        GroupState::PreparingRebalance => {
            SyncOutcome::Immediate(sync_error(ResponseError::RebalanceInProgress.code()))
        }
        GroupState::CompletingRebalance => {
            if group.leader_id.as_deref() == Some(member_id) {
                let assigned: HashMap<String, Bytes> = assignments.into_iter().collect();
                for (id, member) in group.members.iter_mut() {
                    member.assignment = assigned.get(id).cloned().unwrap_or_default();
                }
                group.state = GroupState::Stable;

                let protocol_type = group.protocol_type.clone();
                let protocol_name = group.selected_protocol.clone();
                let own = SyncResult {
                    error_code: 0,
                    protocol_type: protocol_type.clone(),
                    protocol_name: protocol_name.clone(),
                    assignment: group.members[member_id].assignment.clone(),
                };
                let mut wakeups = Vec::new();
                for (id, member) in group.members.iter_mut() {
                    if id == member_id {
                        continue;
                    }
                    if let Some(tx) = member.sync_waiter.take() {
                        wakeups.push((
                            tx,
                            SyncResult {
                                error_code: 0,
                                protocol_type: protocol_type.clone(),
                                protocol_name: protocol_name.clone(),
                                assignment: member.assignment.clone(),
                            },
                        ));
                    }
                }
                SyncOutcome::Completed { own, wakeups }
            } else {
                let (tx, rx) = oneshot::channel();
                group.members.get_mut(member_id).unwrap().sync_waiter = Some(tx);
                SyncOutcome::Park(rx)
            }
        }
        GroupState::Empty | GroupState::Dead => {
            SyncOutcome::Immediate(sync_error(ResponseError::UnknownMemberId.code()))
        }
    }
}

pub(crate) fn expire_sync(
    group: &mut GroupMeta,
    generation_id: i32,
) -> Vec<(oneshot::Sender<SyncResult>, SyncResult)> {
    if group.state != GroupState::CompletingRebalance || group.generation_id != generation_id {
        return Vec::new();
    }

    let mut wakeups = Vec::new();
    for member in group.members.values_mut() {
        if let Some(tx) = member.sync_waiter.take() {
            wakeups.push((tx, sync_error(ResponseError::RebalanceInProgress.code())));
        }
    }
    group.members.clear();
    group.state = GroupState::Empty;
    group.leader_id = None;
    wakeups
}
