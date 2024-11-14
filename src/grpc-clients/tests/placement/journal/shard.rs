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

    use common_base::tools::{get_local_ip, unique_id};
    use grpc_clients::placement::journal::call::{create_shard, list_segment, list_shard};
    use grpc_clients::placement::placement::call::register_node;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::journal::segment::JournalSegment;
    use metadata_struct::journal::shard::{JournalShard, JournalShardStatus};
    use protocol::placement_center::placement_center_inner::{ClusterType, RegisterNodeRequest};
    use protocol::placement_center::placement_center_journal::{
        CreateShardRequest, ListSegmentRequest, ListShardRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn shard_segment_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster_name = unique_id();
        let shard_name = "s1".to_string();
        let namespace = "n1".to_string();
        let node_id = 1;

        // register_node
        let extend = JournalNodeExtend {
            data_fold: vec!["/data1/".to_string()],
            tcp_addr: "".to_string(),
            tcps_addr: "".to_string(),
        };

        let request = RegisterNodeRequest {
            cluster_type: ClusterType::JournalServer.into(),
            cluster_name: cluster_name.clone(),
            node_id,
            node_ip: get_local_ip(),
            node_inner_addr: "127.0.0.1:4531".to_string(),
            extend_info: serde_json::to_string(&extend).unwrap(),
        };
        if let Err(e) = register_node(client_pool.clone(), addrs.clone(), request).await {
            println!("{}", e.to_string());
            assert!(false);
        }

        // create shard
        let request = CreateShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            replica: 1,
        };
        if let Err(e) = create_shard(client_pool.clone(), addrs.clone(), request).await {
            println!("{}", e.to_string());
            assert!(false);
        }

        // list shard
        let request = ListShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        let reply = list_shard(client_pool.clone(), addrs.clone(), request)
            .await
            .unwrap();
        let data: Vec<JournalShard> = serde_json::from_slice(&reply.shards).unwrap();
        assert_eq!(data.len(), 1);
        let shard_raw = data.first().unwrap();
        assert_eq!(shard_raw.cluster_name, cluster_name);
        assert_eq!(shard_raw.namespace, namespace);
        assert_eq!(shard_raw.shard_name, shard_name);
        assert_eq!(shard_raw.replica, 1);
        assert_eq!(shard_raw.start_segment_seq, 0);
        assert_eq!(shard_raw.active_segment_seq, 0);
        assert_eq!(shard_raw.last_segment_seq, 0);
        assert_eq!(shard_raw.status, JournalShardStatus::Run);

        // List Segment
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment(client_pool.clone(), addrs.clone(), request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        println!("{:?}", data);
        assert_eq!(data.len(), 1);
    }
}
