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

    use crate::common::get_placement_addr;
    use common_base::tools::now_second;
    use grpc_clients::meta::common::call::register_node;
    use grpc_clients::meta::mqtt::call::placement_get_share_sub_leader;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_common::RegisterNodeRequest;
    use protocol::meta::meta_service_mqtt::GetShareSubLeaderRequest;

    #[tokio::test]

    async fn mqtt_share_sub_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let group_name: String = "test_group".to_string();
        let node_ip: String = "127.0.0.1".to_string();
        let node_id: u64 = 1;

        let node = BrokerNode {
            roles: Vec::new(),
            node_ip: node_ip.clone(),
            node_id,
            grpc_addr: "127.0.0.1:1228".to_string(),
            extend: Vec::new(),
            register_time: now_second(),
            start_time: now_second(),
            storage_fold: Vec::new(),
            ..Default::default()
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        match register_node(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };

        let request = GetShareSubLeaderRequest {
            group_name: group_name.clone(),
        };
        match placement_get_share_sub_leader(&client_pool, &addrs, request).await {
            Ok(data) => {
                let mut flag = false;
                if data.broker_id == node_id
                    && data.broker_addr == node_ip
                    && data.extend_info.is_empty()
                {
                    flag = true;
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("{e:?}");
            }
        }

        // Test with empty group_name - should fail validation
        let request = GetShareSubLeaderRequest {
            group_name: "".to_string(),
        };
        assert!(
            placement_get_share_sub_leader(&client_pool, &addrs, request)
                .await
                .is_err()
        );
    }
}
