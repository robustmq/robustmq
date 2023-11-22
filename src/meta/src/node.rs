#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeRaftState {
    Leader,
    Follower,
    Candidate,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeState {
    Running,
    Starting,
    Stoping,
    Stop,
}

pub struct Node {
    pub node_ip: String,
    pub node_id: i32,
    pub leader_id: Option<i32>,
    pub leader_ip: Option<String>,
    pub raft_state: NodeRaftState,
    pub state: NodeState,
    pub voter: Option<i32>,
}

impl Node {
    
    pub fn new(node_ip: String, node_id: i32) -> Node {
        Node {
            node_ip: node_ip,
            node_id: node_id,
            leader_id: None,
            leader_ip: None,
            raft_state: NodeRaftState::Candidate,
            state: NodeState::Starting,
            voter: None,
        }
    }

    pub fn update_raft_state(&mut self, raft_state: NodeRaftState) {
        self.raft_state = raft_state
    }

    pub fn update_status(&mut self, state: NodeState) {
        self.state = state;
    }

}
/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */