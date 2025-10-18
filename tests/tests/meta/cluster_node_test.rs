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
    use common_base::tools::now_second;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_inner::meta_service_service_client::MetaServiceServiceClient;
    use protocol::meta::meta_service_inner::{
        NodeListRequest, RegisterNodeRequest, UnRegisterNodeRequest,
    };

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
            extend: extend_info(),
            node_inner_addr: node_ip(),
            start_time: now_second(),
            register_time: now_second(),
        };
        let request = RegisterNodeRequest {
            node: node.encode(),
        };
        client
            .register_node(tonic::Request::new(request))
            .await
            .unwrap();

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
                panic!("{e:?}");
            }
        }
        let request = UnRegisterNodeRequest {
            cluster_name: cluster_name.clone(),
            node_id,
        };
        client
            .un_register_node(tonic::Request::new(request))
            .await
            .unwrap();

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
                panic!("{e:?}");
            }
        }
    }
}
