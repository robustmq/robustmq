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

use super::raft_node::RaftNode;
use common_base::config::placement_center::placement_center_conf;
use dashmap::DashMap;
use raft::StateRole;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ClusterMetadata {
    pub local: RaftNode,
    pub leader: Option<RaftNode>,
    pub raft_role: String,
    pub votes: DashMap<u64, RaftNode>,
    pub members: DashMap<u64, RaftNode>,
}

impl ClusterMetadata {
    pub fn new() -> ClusterMetadata {
        let config = placement_center_conf();
        if config.node.node_id == 0 {
            panic!("node ids can range from 0 to 65536");
        }

        let node_addr = if let Some(addr) =
            config.node.nodes.get(&format!("{}", config.node.node_id))
        {
            addr.to_string()
        } else {
            panic!("node id {} There is no corresponding service address, check the nodes configuration",config.node.node_id);
        };

        let local = RaftNode {
            node_id: config.node.node_id,
            node_addr,
        };

        let votes = DashMap::with_capacity(2);
        for (node_id, addr) in config.node.nodes.clone() {
            let id: u64 = match node_id.to_string().trim().parse() {
                Ok(id) => id,
                Err(_) => {
                    panic!("Node id must be u64");
                }
            };

            if addr.to_string().is_empty() {
                panic!(
                    "Address corresponding to the node id {} cannot be empty",
                    id
                );
            }

            let node = RaftNode {
                node_id: id,
                node_addr: addr.to_string(),
            };

            votes.insert(id, node);
        }

        ClusterMetadata {
            local,
            leader: None,
            raft_role: format!("{:?}", StateRole::Follower),
            votes,
            members: DashMap::with_capacity(2),
        }
    }

    pub fn add_member(&mut self, id: u64, node: RaftNode) {
        self.members.insert(id, node);
    }

    pub fn remove_member(&mut self, id: u64) {
        self.members.remove(&id);
    }

    pub fn update_raft_role(&mut self, local_new_role: StateRole, new_leader: Option<RaftNode>) {
        self.raft_role = format!("{:?}", local_new_role);
        self.leader = new_leader;
    }

    pub fn get_node_by_id(&self, id: u64) -> Option<RaftNode> {
        if let Some(node) = self.votes.get(&id) {
            return Some(node.clone());
        }
        return None;
    }

    pub fn is_raft_role_change(&self, new_role: StateRole) -> bool {
        let n_role = format!("{:?}", new_role);
        return !(n_role == self.raft_role);
    }

    pub fn is_leader(&self) -> bool {
        self.raft_role == format!("{:?}", StateRole::Leader)
    }
}
