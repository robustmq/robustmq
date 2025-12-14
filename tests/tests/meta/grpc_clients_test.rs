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
    use crate::meta::common::{
        cluster_name, extend_info, namespace, node_id, node_ip, pc_addr, shard_name,
    };
    use common_base::tools::now_second;
    use metadata_struct::journal::shard::JournalShardConfig;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_common::meta_service_service_client::MetaServiceServiceClient;
    use protocol::meta::meta_service_common::{
        HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
    };
    use protocol::meta::meta_service_journal::engine_service_client::EngineServiceClient;
    use protocol::meta::meta_service_journal::{
        CreateNextSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest,
    };

    #[tokio::test]
    async fn test_register_node_method_is_ok() {
        let mut client = MetaServiceServiceClient::connect(pc_addr()).await.unwrap();
        let node = BrokerNode {
            roles: Vec::new(),
            node_ip: node_ip(),
            node_id: node_id(),
            node_inner_addr: node_ip(),
            extend: extend_info(),
            register_time: now_second(),
            start_time: now_second(),
        };
        let valid_request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        let response = client
            .register_node(tonic::Request::new(valid_request.clone()))
            .await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_register_node_method_param_ip_is_correct() {
        let client = MetaServiceServiceClient::connect(pc_addr())
            .await
            .expect("Failed to connect to MetaServiceService");

        let mut node = BrokerNode {
            roles: Vec::new(),
            node_ip: node_ip(),
            node_id: node_id(),
            node_inner_addr: node_ip(),
            extend: extend_info(),
            start_time: now_second(),
            register_time: now_second(),
        };
        let valid_request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };

        let valid_ip = [
            "192.168.1.1",
            "1.1.1.1",
            "255.255.255.255",
            "0.0.0.0",
            "123.112.154.123",
            "172.192.140.21",
            "192.168.1.1:8080",
            "1.1.1.1:2020",
            "255.255.255.255:9999",
            "0.0.0.0:10009",
            "123.112.154.123:10000",
            "172.192.140.21:32145",
        ];

        let field = ["node_ip", "node_inner_addr"];

        let mut field_ip: Vec<(&str, &str)> = Vec::new();

        for field in field {
            for ip in valid_ip {
                field_ip.push((field, ip))
            }
        }

        for (field, ip) in field_ip {
            let mut test_client = client.clone();
            match field {
                "node_ip" => node.node_ip = ip.parse().unwrap(),
                "node_inner_addr" => node.node_inner_addr = ip.parse().unwrap(),
                _ => unreachable!(),
            }

            {
                let response = test_client
                    .register_node(tonic::Request::new(valid_request.clone()))
                    .await;
                assert!(response.is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let mut client = MetaServiceServiceClient::connect(pc_addr()).await.unwrap();

        let request = HeartbeatRequest { node_id: node_id() };
        let res = client.heartbeat(tonic::Request::new(request)).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_unregister_node() {
        let mut client = MetaServiceServiceClient::connect(pc_addr()).await.unwrap();

        let request = UnRegisterNodeRequest { node_id: node_id() };
        client
            .un_register_node(tonic::Request::new(request))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let config = JournalShardConfig {
            max_segment_size: 1024 * 1024 * 10,
            replica_num: 1,
        };

        let request = CreateShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
            shard_config: config.encode().unwrap(),
        };

        match client.create_shard(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }

    #[tokio::test]
    async fn test_delete_shard() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = DeleteShardRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        match client.delete_shard(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }

    #[tokio::test]
    async fn test_create_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = CreateNextSegmentRequest {
            namespace: namespace(),
            shard_name: shard_name(),
        };
        match client
            .create_next_segment(tonic::Request::new(request))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }

    #[tokio::test]
    async fn test_delete_segment() {
        let mut client = EngineServiceClient::connect(pc_addr()).await.unwrap();

        let request = DeleteSegmentRequest {
            namespace: namespace(),
            shard_name: shard_name(),
            segment_seq: 1,
        };
        match client.delete_segment(tonic::Request::new(request)).await {
            Ok(_) => {}
            Err(e) => {
                println!("{e}");
            }
        }
    }
}
