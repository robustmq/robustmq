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
    use common_base::role::{ROLE_BROKER, ROLE_ENGINE, ROLE_META};
    use common_base::tools::now_second;
    use grpc_clients::meta::common::call::{placement_list_share_group, register_node};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::meta::extend::NodeExtend;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::mqtt::share_group::ShareGroup;
    use protocol::meta::meta_service_common::{ListShareGroupRequest, RegisterNodeRequest};

    #[tokio::test]
    async fn mqtt_share_sub_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let group_name: String = "test_group".to_string();
        let node_ip: String = "127.0.0.1".to_string();
        let node_id: u64 = 1;

        let node = BrokerNode {
            roles: vec![
                ROLE_BROKER.to_string(),
                ROLE_ENGINE.to_string(),
                ROLE_META.to_string(),
            ],
            node_ip: node_ip.clone(),
            node_id,
            grpc_addr: "127.0.0.1:1228".to_string(),
            extend: NodeExtend::default(),
            register_time: now_second(),
            start_time: now_second(),
            storage_fold: vec!["./data/broker/engine".to_string()],
            engine_addr: "127.0.0.1:1778".to_string(),
        };
        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };
        register_node(&client_pool, &addrs, request).await.unwrap();

        let request = ListShareGroupRequest {
            tenant: "default".to_string(),
            group: group_name.clone(),
        };
        let data = placement_list_share_group(&client_pool, &addrs, request)
            .await
            .unwrap();
        assert_eq!(data.groups.len(), 1);
        let leader = ShareGroup::decode(data.groups.first().unwrap()).unwrap();
        assert_eq!(leader.group_name, group_name);
        assert_eq!(leader.leader_broker, node_id);

        let request = ListShareGroupRequest {
            tenant: "default".to_string(),
            group: String::new(),
        };
        let data = placement_list_share_group(&client_pool, &addrs, request)
            .await
            .unwrap();
        assert!(data.groups.is_empty());
    }
}
