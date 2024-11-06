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
    use common_base::error::common::CommonError;
    use protocol::placement_center::placement_center_inner::placement_center_service_client::PlacementCenterServiceClient;
    use protocol::placement_center::placement_center_inner::{
        HeartbeatRequest, RegisterNodeRequest, SetIdempotentDataRequest, UnRegisterNodeRequest,
    };
    use protocol::placement_center::placement_center_journal::engine_service_client::EngineServiceClient;
    use protocol::placement_center::placement_center_journal::{
        CreateNextSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest,
    };

    use crate::common::{
        cluster_name, cluster_type, extend_info, namespace, node_id, node_ip, pc_addr, producer_id,
        seq_num, shard_name, shard_replica,
    };

    #[tokio::test]
    async fn test_register_node() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let request = RegisterNodeRequest {
            cluster_type: cluster_type(),
            cluster_name: cluster_name(),
            node_id: node_id(),
            node_ip: node_ip(),
            extend_info: extend_info(),
            ..Default::default()
        };
        client
            .register_node(tonic::Request::new(request))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let request = HeartbeatRequest {
            cluster_type: cluster_type(),
            cluster_name: cluster_name(),
            node_id: node_id(),
        };
        client
            .heartbeat(tonic::Request::new(request))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_unregister_node() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let request = UnRegisterNodeRequest {
            cluster_type: cluster_type(),
            cluster_name: cluster_name(),
            node_id: node_id(),
        };
        client
            .un_register_node(tonic::Request::new(request))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_set_idempotent_data() {
        let mut client = PlacementCenterServiceClient::connect(pc_addr())
            .await
            .unwrap();

        let valid_request = SetIdempotentDataRequest {
            cluster_name: cluster_name(),
            producer_id: producer_id(),
            seq_num: seq_num(),
        };

        let response = client
            .set_idempotent_data(tonic::Request::new(valid_request.clone()))
            .await;

        assert!(response.is_ok());

        let invalid_fields = vec![
            ("cluster_name", String::new()),
            ("producer_id", String::new()),
        ];

        for (field, value) in invalid_fields {
            let mut invalid_request = valid_request.clone();
            match field {
                "cluster_name" => invalid_request.cluster_name = value,
                "producer_id" => invalid_request.producer_id = value,
                _ => unreachable!(),
            }

            let response = client
                .set_idempotent_data(tonic::Request::new(invalid_request))
                .await;

            if let Err(ref e) = response {
                assert!(response.is_err());
                assert_eq!(e.code(), tonic::Code::Cancelled);
                assert_eq!(
                    e.message(),
                    CommonError::ParameterCannotBeNull(field.replace("_", " ").to_string())
                        .to_string()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_create_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = CreateShardRequest {
            cluster_name: cluster_name(),
            namespace: namespace(),
            shard_name: shard_name(),
            replica: shard_replica(),
        };
        match client.create_shard(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_delete_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = DeleteShardRequest {
            cluster_name: cluster_name(),
            namespace: namespace(),
            shard_name: shard_name(),
        };
        match client.delete_shard(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = CreateNextSegmentRequest {
            cluster_name: cluster_name(),
            namespace: namespace(),
            shard_name: shard_name(),
            active_segment_next_num: 1,
        };
        match client
            .create_next_segment(tonic::Request::new(request))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_delete_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = DeleteSegmentRequest {
            cluster_name: cluster_name(),
            namespace: namespace(),
            shard_name: shard_name(),
            segment_seq: 1,
        };
        match client.delete_segment(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
