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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use grpc_clients::meta::common::call::{
        placement_openraft_add_learner, placement_openraft_change_membership,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::meta::meta_service_common::{AddLearnerRequest, ChangeMembershipRequest, Node};

    use crate::common::get_placement_addr;

    #[tokio::test]
    #[ignore]
    async fn placement_openraft_add_learner_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let node_id = 2;
        let node = Some(Node {
            rpc_addr: "127.0.0.1:2228".to_string(),
            node_id: 2,
        });
        let blocking = true;

        let request = AddLearnerRequest {
            node_id,
            node: node.clone(),
            blocking,
        };
        match placement_openraft_add_learner(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };
    }

    #[tokio::test]
    #[ignore]
    async fn placement_openraft_change_membership_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let members = vec![1];
        let retain = false;

        let request = ChangeMembershipRequest {
            machine: "t1".to_string(),
            members,
            retain,
        };
        match placement_openraft_change_membership(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };
    }
}
