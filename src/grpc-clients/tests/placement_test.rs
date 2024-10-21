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
    use std::sync::Arc;

    use grpc_clients::placement::placement::call::{
        cluster_status, register_node, unregister_node, delete_idempotent_data
    };
    use grpc_clients::poll::ClientPool;
    use protocol::placement_center::placement_center_inner::{
        ClusterStatusRequest, ClusterType, RegisterNodeRequest, UnRegisterNodeRequest,
        DeleteIdempotentDataRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn register_node_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();

        match cluster_status(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let cluster_type = ClusterType::PlacementCenter as i32;
        let cluster_name = "test-cluster-name".to_string();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 1235u64;
        let node_inner_addr = node_ip.clone();
        let extend_info = "".to_string();

        let request = RegisterNodeRequest {
            cluster_type,
            cluster_name: cluster_name.clone(),
            node_ip: node_ip.clone(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend_info: extend_info.clone(),
        };
        match register_node(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request_cluster_name_empty = RegisterNodeRequest {
            cluster_type,
            cluster_name: "".to_string(),
            node_ip: node_ip.clone(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend_info: extend_info.clone(),
        };
        match register_node(
            client_poll.clone(),
            addrs.clone(),
            request_cluster_name_empty,
        )
        .await
        {
            Ok(_) => {
                panic!("Should not passed because cluster_name is empty");
            }
            Err(_e) => {}
        }

        let request_node_ip_empty = RegisterNodeRequest {
            cluster_type,
            cluster_name: cluster_name.to_string(),
            node_ip: "".to_string(),
            node_id,
            node_inner_addr: node_inner_addr.clone(),
            extend_info: extend_info.clone(),
        };
        match register_node(client_poll.clone(), addrs.clone(), request_node_ip_empty).await {
            Ok(_) => {
                panic!("Should not passed because node_ip is empty");
            }
            Err(_e) => {}
        }
    }

    #[tokio::test]
    async fn un_register_nodet_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(client_poll.clone(), addrs.clone(), request)
            .await
            .is_ok());

        let cluster_type = ClusterType::PlacementCenter as i32;
        let cluster_name = "test-cluster-name".to_string();
        let node_id = 1235u64;

        let request = UnRegisterNodeRequest {
            cluster_type,
            cluster_name: cluster_name.clone(),
            node_id,
        };
        assert!(unregister_node(client_poll.clone(), addrs.clone(), request)
            .await
            .is_ok());

        let request_cluster_name_empty = UnRegisterNodeRequest {
            cluster_type,
            cluster_name: "".to_string(),
            node_id,
        };
        assert!(unregister_node(
            client_poll.clone(),
            addrs.clone(),
            request_cluster_name_empty
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn delete_idempotent_data_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request = ClusterStatusRequest::default();
        assert!(cluster_status(client_poll.clone(), addrs.clone(), request)
            .await
            .is_ok());

        let request = DeleteIdempotentDataRequest {
            cluster_name: "test-cluster-name".to_string(),
            producer_id: "2".to_string(),
            seq_num: 1235u64,
        };
        assert!(
            delete_idempotent_data(client_poll.clone(), addrs.clone(), request)
                .await
                .is_ok()
        );

        let request = DeleteIdempotentDataRequest {
            cluster_name: "".to_string(),
            producer_id: "2".to_string(),
            seq_num: 1235u64,
        };
        assert!(
            delete_idempotent_data(client_poll.clone(), addrs.clone(), request)
                .await
                .is_err()
        );
    }
}
