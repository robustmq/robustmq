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

use crate::{client::find_leader, errors::MetaError, Node};
use common::log::{error_meta, info_meta};
use futures_util::{future::select_ok, FutureExt};

#[derive(Clone)]
pub struct Election {
    local: Node,
    voters: Vec<String>,
}

impl Election {
    pub fn new(local: Node, votes: Vec<String>) -> Self {
        return Election {
            local,
            voters: votes,
        };
    }

    pub async fn leader_election(&self) -> Result<Node, MetaError> {
        //
        let node = match self.find_leader_info().await {
            Ok(nd) => nd,
            Err(_) => {
                info_meta(&format!("Failed to obtain the Leader information from the cluster. An attempt was made to initiate the election process."));

                return self.invite_vote().await;
            }
        };

        return Ok(node);
    }

    // Obtain the cluster Leader information from multiple nodes in parallel
    async fn find_leader_info(&self) -> Result<Node, MetaError> {
        let mut futs = Vec::new();
        for addr in &self.voters {
            if addr.to_string() == self.local.addr() {
                continue;
            }
            let fut = async move { find_leader(&addr).await };
            futs.push(fut.boxed());
        }

        match select_ok(futs).await {
            Ok(reply) => {
                return Ok(Node::new(
                    reply.0.leader_ip,
                    reply.0.leader_id,
                    reply.0.leader_port as u16,
                ))
            }
            Err(err) => {
                error_meta(&format!("Failed to obtain Leader information from another node during the election. error message: {}",err));
            }
        }
        return Err(MetaError::MetaClusterNotLeaderNode);
    }

    async fn invite_vote(&self) -> Result<Node, MetaError> {
        return Ok(self.local.clone());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_leader_info() {}
}
