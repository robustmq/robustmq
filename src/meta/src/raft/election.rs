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

use crate::{errors::MetaError, Node, client::find_leader};
use common::log::error_meta;

#[derive(Clone)]
pub struct Election {
    voters: Vec<String>,
}

impl Election {
    pub fn new(votes: Vec<String>) -> Self {
        return Election { voters: votes };
    }

    pub async fn leader_election(&self) -> Result<Node, MetaError> {
        // 
        let node = match self.find_leader_info().await {
            Ok(nd) => nd,
            Err(err) => {
                error_meta(&format!(
                    "Failed to obtain the Leader from another node. 
                    The voting process starts. Error message: {}",
                    err
                ));

                return self.invite_vote().await;
            }
        };

        return Ok(node);
    }

    async fn find_leader_info(&self) -> Result<Node, MetaError> {
        for addr in &self.voters {
            match find_leader(&addr).await {
                Ok(reply) => return Ok(Node::new(reply.leader_ip, reply.leader_id)),
                Err(err) => {
                    error_meta(&format!("Failed to obtain Leader information from another node during the election. 
                    The IP address of the target node is {}, and the error message is {}",addr,err));
                    continue;
                }
            };
        }
        return Err(MetaError::MetaClusterNotLeaderNode);
    }

    async fn invite_vote(&self) -> Result<Node, MetaError> {
        return Ok(Node::new("".to_string(), 1));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn leader_election() {}
}
