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

mod common;

#[cfg(test)]
mod tests {
    use metadata_struct::placement::broker_node::BrokerNode;
    use protocol::placement_center::generate::placement::placement_center_service_client::PlacementCenterServiceClient;
    use protocol::placement_center::generate::placement::{
        NodeListRequest, RegisterNodeRequest, UnRegisterNodeRequest,
    };

    use crate::common::{cluster_name, cluster_type, extend_info, node_id, node_ip, pc_addr};

    #[tokio::test]
    async fn node_heartbeat_keep_alive_test() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();
        let cluster_name = cluster_name();
        let node_id = node_id();
        let mut request = RegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name.clone();
        request.node_id = node_id;
        request.node_ip = node_ip();
        request.extend_info = extend_info();
        match client.register_node(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }

        let request = NodeListRequest {
            cluster_name: cluster_name.clone(),
        };
        match client.node_list(request).await {
            Ok(rep) => {
                let mut flag = false;
                let nodes = rep.into_inner().nodes;
                for raw in nodes {
                    let node = serde_json::from_slice::<BrokerNode>(&raw).unwrap();
                    if node.node_id == node_id {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
        let mut request = UnRegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name.clone();
        request.node_id = node_id;
        match client.un_register_node(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }

        let request = NodeListRequest {
            cluster_name: cluster_name.clone(),
        };
        match client.node_list(request).await {
            Ok(rep) => {
                let mut flag = false;
                let nodes = rep.into_inner().nodes;
                for raw in nodes {
                    let node = serde_json::from_slice::<BrokerNode>(&raw).unwrap();
                    if node.node_id == node_id {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }
}
