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
    use std::thread::sleep;
    use std::time::Duration;

    use common_base::tools::unique_id;
    use grpc_clients::placement::mqtt::call::placement_get_share_sub_leader;
    use grpc_clients::placement::placement::call::register_node;
    use grpc_clients::poll::ClientPool;
    use log::info;
    use protocol::placement_center::placement_center_inner::{ClusterType, RegisterNodeRequest};
    use protocol::placement_center::placement_center_mqtt::GetShareSubLeaderRequest;

    #[tokio::test]
    async fn test_share_sub() {
        let client_poll = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:1228".to_string()];

        let cluster_type = ClusterType::MqttBrokerServer.into();
        let cluster_name = unique_id();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 7;
        let node_inner_addr = "127.0.0.1:8228".to_string();
        let extend_info = "".to_string();
        let request = RegisterNodeRequest {
            cluster_type,
            cluster_name: cluster_name.clone(),
            node_ip,
            node_id,
            node_inner_addr,
            extend_info,
        };

        sleep(Duration::from_secs(2));

        let res = register_node(client_poll.clone(), addrs.clone(), request)
            .await
            .unwrap();
        info!("{:?}", res);

        let group_name = "test".to_string();
        let req = GetShareSubLeaderRequest {
            cluster_name: cluster_name.clone(),
            group_name,
        };

        let resp = placement_get_share_sub_leader(client_poll.clone(), addrs.clone(), req)
            .await
            .unwrap();
        println!("resp broker_id:{}", resp.broker_id);
        println!("node_id:{}", node_id);
        assert_eq!(resp.broker_id, node_id);
    }
}
