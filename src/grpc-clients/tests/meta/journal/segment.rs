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
    use grpc_clients::meta::journal::call::{create_next_segment, create_shard};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::journal::shard::JournalShardConfig;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::meta::meta_service_common::RegisterNodeRequest;
    use protocol::meta::meta_service_journal::{CreateNextSegmentRequest, CreateShardRequest};

    use crate::common::get_placement_addr;
    #[tokio::test]
    async fn segment_test() {
        let client_pool = ClientPool::new(1);
        let addrs = vec![get_placement_addr()];

        let namespace = "n1".to_string();
        let shard_name = "s1".to_string();

        // register_node
        let extend = JournalNodeExtend {
            tcp_addr: "".to_string(),
            data_fold: vec!["/data".to_string()],
        };

        let node = BrokerNode {
            roles: Vec::new(),
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            node_inner_addr: "127.0.0.1:3228".to_string(),
            extend: extend.encode().unwrap(),
            start_time: now_second(),
            register_time: now_second(),
        };

        let request = RegisterNodeRequest {
            node: node.encode().unwrap(),
        };

        register_node(&client_pool, &addrs, request).await.unwrap();

        let config = JournalShardConfig {
            replica_num: 1,
            max_segment_size: 10 * 1024 * 1024,
        };
        //  create shard
        let request = CreateShardRequest {
            namespace: namespace.clone(),
            shard_name: shard_name.clone(),
            shard_config: config.encode().unwrap(),
        };
        let res = create_shard(&client_pool, &addrs, request).await.unwrap();
        assert_eq!(res.replica.len(), 1);
        assert_eq!(res.segment_no, 0);

        let request = CreateNextSegmentRequest {
            namespace: namespace.clone(),
            shard_name: shard_name.clone(),
        };
        create_next_segment(&client_pool, &addrs, request)
            .await
            .unwrap();
    }
}
