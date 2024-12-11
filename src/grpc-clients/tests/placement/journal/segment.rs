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

    use common_base::tools::unique_id;
    use grpc_clients::placement::inner::call::register_node;
    use grpc_clients::placement::journal::call::{create_next_segment, create_shard};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use protocol::placement_center::placement_center_inner::{ClusterType, RegisterNodeRequest};
    use protocol::placement_center::placement_center_journal::{
        CreateNextSegmentRequest, CreateShardRequest,
    };

    use crate::common::get_placement_addr;
    #[tokio::test]
    async fn segment_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster = unique_id();
        let namespace = "n1".to_string();
        let shard_name = "s1".to_string();

        // register_node
        let extend = JournalNodeExtend {
            tcp_addr: "".to_string(),
            tcps_addr: "".to_string(),
            data_fold: vec!["/data".to_string()],
        };
        let request = RegisterNodeRequest {
            cluster_type: ClusterType::JournalServer.into(),
            cluster_name: cluster.clone(),
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            node_inner_addr: "127.0.0.1:3228".to_string(),
            extend_info: serde_json::to_string(&extend).unwrap(),
        };
        register_node(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        //  create shard
        let request = CreateShardRequest {
            cluster_name: cluster.clone(),
            namespace: namespace.clone(),
            shard_name: shard_name.clone(),
            replica: 1,
        };
        let res = create_shard(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        assert_eq!(res.replica.len(), 1);
        assert_eq!(res.segment_no, 0);

        let request = CreateNextSegmentRequest {
            cluster_name: cluster.clone(),
            namespace: namespace.clone(),
            shard_name: shard_name.clone(),
        };
        create_next_segment(client_pool, &addrs, request)
            .await
            .unwrap();
    }
}
