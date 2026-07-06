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

use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use common_base::uuid::unique_id;
use kafka_protocol::error::ResponseError;
use tokio::sync::oneshot;

use crate::core::group_meta::{GroupMeta, GroupState, MemberMeta};
use crate::core::sync::{sync_error, SyncResult};

#[derive(Debug, Clone)]
pub struct JoinResult {
    pub error_code: i16,
    pub generation_id: i32,
    pub protocol_type: Option<String>,
    pub protocol_name: Option<String>,
    pub leader_id: String,
    pub member_id: String,
    pub members: Vec<(String, Bytes)>,
}

#[derive(Default)]
pub struct JoinCompletion {
    pub deliveries: Vec<(oneshot::Sender<JoinResult>, JoinResult)>,
    pub completing_generation: Option<i32>,
}

pub enum JoinTimer {
    // Brand-new group: batch initial joiners for the configured delay.
    InitialDelay,
    // Rebalance of an existing group: wait up to the members' rebalance timeout.
    RebalanceTimeout(u64),
}

pub struct AddMemberOutcome {
    pub join_rx: oneshot::Receiver<JoinResult>,
    pub timer: Option<JoinTimer>,
    // Every known member has rejoined: the join phase can complete immediately.
    pub complete_now: bool,
    pub sync_wakeups: Vec<(oneshot::Sender<SyncResult>, SyncResult)>,
}

pub struct JoinGroupParams {
    pub group_id: String,
    pub member_id: String,
    pub group_instance_id: Option<String>,
    pub client_id: String,
    pub session_timeout_ms: i32,
    pub rebalance_timeout_ms: i32,
    pub protocol_type: String,
    pub protocols: Vec<(String, Bytes)>,
    pub require_member_id: bool,
}

pub(crate) fn generate_member_id(client_id: &str) -> String {
    format!("{}-{}", client_id, unique_id())
}

pub(crate) fn join_error(error_code: i16, member_id: String) -> JoinResult {
    JoinResult {
        error_code,
        generation_id: -1,
        protocol_type: None,
        protocol_name: None,
        leader_id: String::new(),
        member_id,
        members: Vec::new(),
    }
}

pub(crate) fn register_member(group: &mut GroupMeta, mut member: MemberMeta) -> AddMemberOutcome {
    let (tx, join_rx) = oneshot::channel();
    member.join_waiter = Some(tx);

    let new_group = group.state == GroupState::Empty && group.members.is_empty();

    // Members parked in SyncGroup must not outlive the rebalance this join starts.
    let mut sync_wakeups = Vec::new();
    if group.state == GroupState::CompletingRebalance {
        for m in group.members.values_mut() {
            if let Some(tx) = m.sync_waiter.take() {
                sync_wakeups.push((tx, sync_error(ResponseError::RebalanceInProgress.code())));
            }
        }
    }

    group.state = GroupState::PreparingRebalance;
    if group.leader_id.is_none() {
        group.leader_id = Some(member.member_id.clone());
    }
    group.members.insert(member.member_id.clone(), member);

    // A rebalance of an existing group completes as soon as every known member has
    // rejoined; a new group instead batches joiners for the initial delay.
    let complete_now = !new_group && group.members.values().all(|m| m.join_waiter.is_some());

    let mut timer = None;
    if complete_now {
        group.rebalance_timer_running = false;
    } else if !group.rebalance_timer_running {
        group.rebalance_timer_running = true;
        timer = Some(if new_group {
            JoinTimer::InitialDelay
        } else {
            let max_rebalance_timeout = group
                .members
                .values()
                .map(|m| m.rebalance_timeout_ms.max(0) as u64)
                .max()
                .unwrap_or(0);
            JoinTimer::RebalanceTimeout(max_rebalance_timeout)
        });
    }

    AddMemberOutcome {
        join_rx,
        timer,
        complete_now,
        sync_wakeups,
    }
}

pub(crate) fn complete_join(group: &mut GroupMeta) -> JoinCompletion {
    group.rebalance_timer_running = false;
    if group.state != GroupState::PreparingRebalance {
        return JoinCompletion::default();
    }

    // Members that did not (re)join this rebalance are dropped, mirroring Kafka:
    // keeping them would elect unreachable leaders and stall every generation.
    group.members.retain(|_, m| m.join_waiter.is_some());

    if group.members.is_empty() {
        group.state = GroupState::Empty;
        group.leader_id = None;
        return JoinCompletion::default();
    }

    let mut deliveries: Vec<(oneshot::Sender<JoinResult>, JoinResult)> = Vec::new();
    let mut completing_generation = None;
    match finalize_generation(group) {
        Ok(()) => {
            completing_generation = Some(group.generation_id);
            let leader_id = group.leader_id.clone().unwrap_or_default();
            let generation_id = group.generation_id;
            let protocol_type = group.protocol_type.clone();
            let protocol_name = group.selected_protocol.clone();
            let leader_members = leader_member_metadata(group, protocol_name.as_deref());

            for (member_id, member) in group.members.iter_mut() {
                let Some(tx) = member.join_waiter.take() else {
                    continue;
                };
                let is_leader = *member_id == leader_id;
                deliveries.push((
                    tx,
                    JoinResult {
                        error_code: 0,
                        generation_id,
                        protocol_type: protocol_type.clone(),
                        protocol_name: protocol_name.clone(),
                        leader_id: leader_id.clone(),
                        member_id: member_id.clone(),
                        members: if is_leader {
                            leader_members.clone()
                        } else {
                            Vec::new()
                        },
                    },
                ));
            }
        }
        Err(error_code) => {
            for (member_id, member) in group.members.iter_mut() {
                if let Some(tx) = member.join_waiter.take() {
                    deliveries.push((tx, join_error(error_code, member_id.clone())));
                }
            }
            group.members.clear();
            group.state = GroupState::Empty;
            group.leader_id = None;
        }
    }
    JoinCompletion {
        deliveries,
        completing_generation,
    }
}

fn finalize_generation(group: &mut GroupMeta) -> Result<(), i16> {
    let protocol_types: HashSet<&str> = group
        .members
        .values()
        .map(|m| m.protocol_type.as_str())
        .collect();
    if protocol_types.len() != 1 {
        return Err(ResponseError::InconsistentGroupProtocol.code());
    }
    let protocol_type = protocol_types.into_iter().next().unwrap().to_string();

    let selected =
        select_protocol(&group.members).ok_or(ResponseError::InconsistentGroupProtocol.code())?;

    let leader_id = match &group.leader_id {
        Some(l) if group.members.contains_key(l) => l.clone(),
        _ => group.members.keys().min().cloned().unwrap(),
    };

    group.generation_id += 1;
    group.protocol_type = Some(protocol_type);
    group.selected_protocol = Some(selected);
    group.leader_id = Some(leader_id);
    group.state = GroupState::CompletingRebalance;
    Ok(())
}

fn select_protocol(members: &HashMap<String, MemberMeta>) -> Option<String> {
    let mut common: Option<HashSet<String>> = None;
    for member in members.values() {
        let names: HashSet<String> = member.protocols.iter().map(|(n, _)| n.clone()).collect();
        common = Some(match common {
            None => names,
            Some(c) => c.intersection(&names).cloned().collect(),
        });
    }
    let common = common.unwrap_or_default();
    if common.is_empty() {
        return None;
    }

    let mut votes: HashMap<String, usize> = HashMap::new();
    for member in members.values() {
        if let Some((name, _)) = member.protocols.iter().find(|(n, _)| common.contains(n)) {
            *votes.entry(name.clone()).or_default() += 1;
        }
    }
    let mut ranked: Vec<(String, usize)> = votes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.into_iter().next().map(|(name, _)| name)
}

fn leader_member_metadata(group: &GroupMeta, selected: Option<&str>) -> Vec<(String, Bytes)> {
    let mut out: Vec<(String, Bytes)> = group
        .members
        .iter()
        .map(|(id, m)| (id.clone(), m.metadata_for(selected)))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}
