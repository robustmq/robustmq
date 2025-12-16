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
    use grpc_clients::meta::common::call::register_node;
    use grpc_clients::meta::mqtt::call::placement_get_share_sub_leader;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_common::RegisterNodeRequest;
    use protocol::meta::meta_service_mqtt::GetShareSubLeaderRequest;
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;
    use tracing::info;

    #[tokio::test]
    async fn test_share_sub() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec!["127.0.0.1:1228".to_string()];
        let node_ip = "127.0.0.1".to_string();
        let node_id = 7;
        let node_inner_addr = "127.0.0.1:8228".to_string();
        let extend_info = Vec::new();
        let node = BrokerNode {
            roles: Vec::new(),
            node_ip,
            node_id,
            node_inner_addr,
            extend: extend_info.clone(),
            register_time: now_second(),
            start_time: now_second(),
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };

        sleep(Duration::from_secs(2));

        let res = register_node(&client_pool, &addrs, request).await.unwrap();
        info!("{:?}", res);

        let group_name = "test".to_string();
        let req = GetShareSubLeaderRequest { group_name };

        let resp = placement_get_share_sub_leader(&client_pool, &addrs, req)
            .await
            .unwrap();
        println!("resp broker_id:{}", resp.broker_id);
        println!("node_id:{node_id}");
        assert_eq!(resp.broker_id, node_id);
    }

}
