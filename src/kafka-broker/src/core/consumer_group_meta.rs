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

use uuid::Uuid;

pub const DEFAULT_ASSIGNOR: &str = "range";

// KIP-848 exchanges topic ids as 16-byte UUIDs, but our Topic.topic_id is an xid
// string. Derive a stable UUID (v5, name-based) from tenant/topic so the id is
// deterministic across brokers and restarts without touching stored metadata.
pub fn topic_uuid(tenant: &str, topic_name: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("{}/{}", tenant, topic_name).as_bytes(),
    )
}

// member_id -> (topic_id -> partitions)
pub type TargetAssignment = HashMap<String, HashMap<Uuid, Vec<i32>>>;

pub struct ConsumerGroupMeta {
    pub group_id: String,
    // Bumped on every membership/subscription change.
    pub group_epoch: i32,
    // The group_epoch the current target assignment was computed at.
    pub assignment_epoch: i32,
    pub assignor: String,
    pub members: HashMap<String, ConsumerMemberMeta>,
    pub target: TargetAssignment,
    // Current owner of each partition, maintained from member reports; a partition
    // is only granted to its target member once the previous owner released it.
    pub owner: HashMap<(Uuid, i32), String>,
}

impl ConsumerGroupMeta {
    pub fn new(group_id: String) -> Self {
        ConsumerGroupMeta {
            group_id,
            group_epoch: 0,
            assignment_epoch: 0,
            assignor: DEFAULT_ASSIGNOR.to_string(),
            members: HashMap::new(),
            target: HashMap::new(),
            owner: HashMap::new(),
        }
    }

    pub fn state_name(&self) -> &'static str {
        if self.members.is_empty() {
            return "Empty";
        }
        let converged = self.members.values().all(|m| {
            m.member_epoch == self.group_epoch && m.reported == self.member_target(&m.member_id)
        });
        if converged {
            "Stable"
        } else {
            "Reconciling"
        }
    }

    pub fn member_target(&self, member_id: &str) -> HashMap<Uuid, Vec<i32>> {
        self.target.get(member_id).cloned().unwrap_or_default()
    }
}

pub struct ConsumerMemberMeta {
    pub member_id: String,
    pub instance_id: Option<String>,
    pub rack_id: Option<String>,
    pub client_id: String,
    pub rebalance_timeout_ms: i32,
    pub subscribed: Vec<String>,
    // What the member last reported it actually owns.
    pub reported: HashMap<Uuid, Vec<i32>>,
    pub member_epoch: i32,
    // Last assignment sent, to omit `assignment` from responses when unchanged.
    pub last_sent: Option<HashMap<Uuid, Vec<i32>>>,
    pub last_heartbeat_ms: u128,
}

pub struct ConsumerDescribedMember {
    pub member_id: String,
    pub instance_id: Option<String>,
    pub rack_id: Option<String>,
    pub client_id: String,
    pub member_epoch: i32,
    pub subscribed: Vec<String>,
    pub assignment: HashMap<Uuid, Vec<i32>>,
    pub target_assignment: HashMap<Uuid, Vec<i32>>,
}

pub struct ConsumerDescribedGroup {
    pub group_id: String,
    pub state: String,
    pub group_epoch: i32,
    pub assignment_epoch: i32,
    pub assignor: String,
    pub members: Vec<ConsumerDescribedMember>,
    // topic_id -> topic_name for every subscribed topic, for responses that carry both.
    pub topic_names: HashMap<Uuid, String>,
}

pub(crate) fn describe(group: &ConsumerGroupMeta, tenant: &str) -> ConsumerDescribedGroup {
    let mut topic_names: HashMap<Uuid, String> = HashMap::new();
    let mut members: Vec<ConsumerDescribedMember> = group
        .members
        .values()
        .map(|m| {
            for name in &m.subscribed {
                topic_names.insert(topic_uuid(tenant, name), name.clone());
            }
            ConsumerDescribedMember {
                member_id: m.member_id.clone(),
                instance_id: m.instance_id.clone(),
                rack_id: m.rack_id.clone(),
                client_id: m.client_id.clone(),
                member_epoch: m.member_epoch,
                subscribed: m.subscribed.clone(),
                assignment: m.reported.clone(),
                target_assignment: group.member_target(&m.member_id),
            }
        })
        .collect();
    members.sort_by(|a, b| a.member_id.cmp(&b.member_id));

    ConsumerDescribedGroup {
        group_id: group.group_id.clone(),
        state: group.state_name().to_string(),
        group_epoch: group.group_epoch,
        assignment_epoch: group.assignment_epoch,
        assignor: group.assignor.clone(),
        members,
        topic_names,
    }
}
