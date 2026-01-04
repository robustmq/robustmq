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
    use crate::common::get_placement_addr;
    use grpc_clients::meta::common::call::{
        cluster_status, delete_resource_config, get_resource_config, node_list, set_resource_config,
    };
    use grpc_clients::pool::ClientPool;
    use protocol::meta::meta_service_common::{
        ClusterStatusRequest, DeleteResourceConfigRequest, GetResourceConfigRequest,
        NodeListRequest, SetResourceConfigRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn node_list_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let request = NodeListRequest {};
        assert!(node_list(&client_pool, &addrs, request).await.is_ok());
    }

    #[tokio::test]
    async fn set_resource_config_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(&client_pool, &addrs, request).await.is_ok());

        let config = vec![1, 2, 3];
        let resources = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        let request = SetResourceConfigRequest {
            resources: resources.clone(),
            config: config.clone(),
        };
        assert!(set_resource_config(&client_pool, &addrs, request)
            .await
            .is_ok());

        let request_empty_resources = SetResourceConfigRequest {
            resources: Vec::new(),
            config,
        };
        assert!(
            set_resource_config(&client_pool, &addrs, request_empty_resources)
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
        let config = vec![1, 2, 3];
        let resources = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        // Set the resource config first
        let set_request = SetResourceConfigRequest {
            resources: resources.clone(),
            config: config.clone(),
        };
        assert!(set_resource_config(&client_pool, &addrs, set_request)
            .await
            .is_ok());

        // Test: Get the resource config
        let valid_get_request = GetResourceConfigRequest {
            resources: resources.clone(),
        };
        assert!(get_resource_config(&client_pool, &addrs, valid_get_request)
            .await
            .is_ok());

        // Test: Get the resource config with empty resources - should fail validation
        let get_request_with_empty_resources = GetResourceConfigRequest {
            resources: Vec::new(),
        };
        assert!(
            get_resource_config(&client_pool, &addrs, get_request_with_empty_resources)
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
            resources: vec!["test".to_string()],
        };
        assert!(delete_resource_config(&client_pool, &addrs, request)
            .await
            .is_ok());

        // Test with empty resources - should fail validation
        let request_empty_resources = DeleteResourceConfigRequest {
            resources: Vec::new(),
        };
        assert!(
            delete_resource_config(&client_pool, &addrs, request_empty_resources)
                .await
                .is_err()
        );
    }
}
