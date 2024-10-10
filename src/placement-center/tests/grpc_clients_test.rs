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
    use protocol::placement_center::generate::journal::engine_service_client::EngineServiceClient;
    use protocol::placement_center::generate::journal::{
        CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest,
    };
    use protocol::placement_center::generate::placement::placement_center_service_client::PlacementCenterServiceClient;
    use protocol::placement_center::generate::placement::{
        HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
    };

    use crate::common::{
        cluster_name, cluster_type, extend_info, node_id, node_ip, pc_addr, shard_name,
        shard_replica,
    };

    #[tokio::test]
    async fn test_register_node() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let mut request = RegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        request.node_ip = node_ip();
        request.extend_info = extend_info();
        match client.register_node(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let mut request = HeartbeatRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        match client.heartbeat(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_unregister_node() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let mut request = UnRegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        match client.un_register_node(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_create_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let mut request = CreateShardRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        request.replica = shard_replica();
        match client.create_shard(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_delete_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let mut request = DeleteShardRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        match client.delete_shard(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_create_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let mut request = CreateSegmentRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        match client.create_segment(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn test_delete_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let mut request = DeleteSegmentRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        request.segment_seq = 1;
        match client.delete_segment(tonic::Request::new(request)).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }
}
