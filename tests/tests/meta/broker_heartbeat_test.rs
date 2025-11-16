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
    use std::time::Duration;

    use common_base::tools::now_second;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_common::meta_service_service_client::MetaServiceServiceClient;
    use protocol::meta::meta_service_common::{NodeListRequest, RegisterNodeRequest};
    use tokio::time::sleep;

    use crate::meta::common::{cluster_name, extend_info, node_id, node_ip, pc_addr};

    #[tokio::test]
    async fn node_heartbeat_keep_alive_test() {
        let mut client = MetaServiceServiceClient::connect(pc_addr()).await.unwrap();
        let cluster_name = cluster_name();
        let node_id = node_id();
        let node = BrokerNode {
            roles: Vec::new(),
            cluster_name: cluster_name.clone(),
            node_id,
            node_ip: node_ip(),
            node_inner_addr: node_ip(),
            extend: extend_info(),
            register_time: now_second(),
            start_time: now_second(),
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        client
            .register_node(tonic::Request::new(request))
            .await
            .unwrap();
        let start_time = now_second();
        loop {
            let request = NodeListRequest {
                cluster_name: cluster_name.clone(),
            };
            let resp = client.node_list(request).await.unwrap();
            let mut flag = false;
            let nodes = resp.into_inner().nodes;
            for raw in nodes {
                let node = BrokerNode::decode(&raw).unwrap();
                if node.node_id == node_id {
                    flag = true;
                }
            }
            if !flag {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        let total_ms = now_second() - start_time;
        println!("{total_ms}");
        assert!((28..=31).contains(&total_ms));
    }
}
