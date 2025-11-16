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

    use common_base::tools::now_second;
    use grpc_clients::meta::inner::call::{
        cluster_status, delete_resource_config, get_resource_config, node_list, register_node,
        set_resource_config, unregister_node,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_inner::{
        ClusterStatusRequest, DeleteResourceConfigRequest, GetResourceConfigRequest,
        NodeListRequest, RegisterNodeRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn register_node_test_is_normal() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();

        match cluster_status(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let cluster_name = "test-cluster-name".to_string();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 1235u64;
        let node_inner_addr = node_ip.clone();
        let extend_info = Vec::new();

        let node = BrokerNode {
            roles: Vec::new(),
            cluster_name: cluster_name.clone(),
            node_ip: node_ip.clone(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend: extend_info.clone(),
            register_time: now_second(),
            start_time: now_second(),
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };

        match register_node(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Should not passed because cluster_name is empty")]
    async fn register_node_test_is_cluster_name_is_empty() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();

        match cluster_status(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let node_ip = "127.0.0.1".to_string();
        let node_id = 1235u64;
        let node_inner_addr = node_ip.clone();
        let extend_info = Vec::new();

        let node = BrokerNode {
            roles: Vec::new(),
            cluster_name: "".to_string(),
            node_ip: node_ip.clone(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend: extend_info.clone(),
            register_time: now_second(),
            start_time: now_second(),
        };

        let request_cluster_name_empty = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };

        match register_node(&client_pool, &addrs, request_cluster_name_empty).await {
            Ok(_) => {
                panic!("Should not passed because cluster_name is empty");
            }
            Err(_e) => {}
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Should not passed because node_ip is empty")]
    async fn register_node_test_is_node_ip_is_empty() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();

        match cluster_status(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let cluster_name = "test-cluster-name".to_string();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 1235u64;
        let node_inner_addr = node_ip.clone();
        let extend_info = Vec::new();

        let node = BrokerNode {
            roles: Vec::new(),
            cluster_name: cluster_name.to_string(),
            node_ip: "".to_string(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend: extend_info.clone(),
            register_time: now_second(),
            start_time: now_second(),
        };
        let request_node_ip_empty = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        match register_node(&client_pool, &addrs, request_node_ip_empty).await {
            Ok(_) => {
                panic!("Should not passed because node_ip is empty");
            }
            Err(_e) => {}
        }
    }

    #[tokio::test]
    async fn un_register_node_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let cluster_name = "test-cluster-name".to_string();
        let node_id = 1235u64;

        let request = UnRegisterNodeRequest {
            cluster_name: cluster_name.clone(),
            node_id,
        };
        assert!(unregister_node(&client_pool, &addrs, request).await.is_ok());

        let request_cluster_name_empty = UnRegisterNodeRequest {
            cluster_name: "".to_string(),
            node_id,
        };
        assert!(
            unregister_node(&client_pool, &addrs, request_cluster_name_empty)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn node_list_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let cluster_name = "test-cluster-name".to_string();

        let request = NodeListRequest {
            cluster_name: cluster_name.clone(),
        };
        assert!(node_list(&client_pool, &addrs, request).await.is_ok());

        let request_cluster_name_empty = NodeListRequest {
            cluster_name: "".to_string(),
        };
        assert!(node_list(&client_pool, &addrs, request_cluster_name_empty)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn set_resource_config_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let cluster_name = "test-cluster-name".to_string();
        let config = vec![1, 2, 3];
        let resources = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        let request = SetResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources: resources.clone(),
            config: config.clone(),
        };
        assert!(set_resource_config(&client_pool, &addrs, request)
            .await
            .is_ok());

        let request_cluster_name_empty = SetResourceConfigRequest {
            cluster_name: "".to_string(),
            resources,
            config,
        };
        assert!(
            set_resource_config(&client_pool, &addrs, request_cluster_name_empty)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn get_resource_config_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        // Check if the cluster is available
        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        // Test data
        let cluster_name = "test-cluster-name".to_string();
        let config = vec![1, 2, 3];
        let resources = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        // Set the resource config first
        let set_request = SetResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources: resources.clone(),
            config: config.clone(),
        };
        assert!(set_resource_config(&client_pool, &addrs, set_request)
            .await
            .is_ok());

        // Test: Get the resource config
        let valid_get_request = GetResourceConfigRequest {
            cluster_name: cluster_name.clone(),
            resources: resources.clone(),
        };
        assert!(get_resource_config(&client_pool, &addrs, valid_get_request)
            .await
            .is_ok());

        // Test: Get the resource config with empty cluster name
        let get_request_with_empty_cluster_name = GetResourceConfigRequest {
            cluster_name: "".to_string(),
            resources,
        };
        assert!(
            get_resource_config(&client_pool, &addrs, get_request_with_empty_cluster_name)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn delete_resource_config_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let request = DeleteResourceConfigRequest {
            cluster_name: "test-cluster-name".to_string(),
            resources: Vec::new(),
        };
        assert!(delete_resource_config(&client_pool, &addrs, request)
            .await
            .is_ok());

        let request = DeleteResourceConfigRequest {
            cluster_name: "".to_string(),
            resources: Vec::new(),
        };
        assert!(delete_resource_config(&client_pool, &addrs, request)
            .await
            .is_err());
    }
}
