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
use crate::errors::MetaError;
use super::{client::find_leader, node::Node};

pub struct Election {
    voters: Vec<String>,
}

impl Election {
    pub fn new(votes: Vec<String>) -> Self {
        return Election { voters: votes };
    }

    pub async fn leader_election(&self) -> Result<Node, MetaError> {
        let mut leader_node: Node = Node::new("".to_string(), 0);

        for addr in &self.voters {
            let res = find_leader(addr).await;
            if leader_node.node_id != 0 && leader_node.leader_id.unwrap() != res.leader_id {
                return Err(MetaError::MultipleLeaders(
                    leader_node.node_ip,
                    res.leader_ip,
                ));
            }
            leader_node = Node::new(res.leader_ip, res.leader_id);
        }
        return Ok(leader_node);
    }

    fn find_leader_info() {}

    fn election() {}
}
