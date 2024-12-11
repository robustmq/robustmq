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
    use grpc_clients::placement::inner::call::register_node;
    use grpc_clients::placement::journal::call::{
        create_next_segment, create_shard, delete_segment, delete_shard, list_segment,
        list_segment_meta, list_shard, update_segment_meta, update_segment_status,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::journal::node_extend::JournalNodeExtend;
    use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
    use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
    use metadata_struct::journal::shard::{JournalShard, JournalShardStatus};
    use protocol::placement_center::placement_center_inner::{ClusterType, RegisterNodeRequest};
    use protocol::placement_center::placement_center_journal::{
        CreateNextSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest,
        ListSegmentMetaRequest, ListSegmentRequest, ListShardRequest, UpdateSegmentMetaRequest,
        UpdateSegmentStatusRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]

    async fn shard_segment_metadata_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster_name = unique_id();
        let shard_name = "s1".to_string();
        let namespace = "n1".to_string();
        let node_id = 1;

        // register_node
        let node_fold = "/data1/".to_string();
        let extend = JournalNodeExtend {
            data_fold: vec![node_fold.clone()],
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
        if let Err(e) = register_node(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // create shard
        let request = CreateShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            replica: 1,
        };
        if let Err(e) = create_shard(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // list shard
        let request = ListShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        let reply = list_shard(client_pool.clone(), &addrs, request)
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
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let segment = data.first().unwrap();
        assert_eq!(segment.namespace, namespace);
        assert_eq!(segment.shard_name, shard_name);
        assert_eq!(segment.segment_seq, 0);
        assert_eq!(segment.leader_epoch, 0);
        assert_eq!(segment.leader, node_id);
        assert_eq!(segment.status, SegmentStatus::Write);
        assert_eq!(segment.config.max_segment_size, 1073741824);
        assert_eq!(segment.replicas.len(), 1);
        let rep = segment.replicas.first().unwrap();
        assert_eq!(rep.replica_seq, 0);
        assert_eq!(rep.node_id, node_id);
        assert_eq!(rep.fold, node_fold);

        // list segment meta
        let request = ListSegmentMetaRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment_meta(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        let data: Vec<JournalSegmentMetadata> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let meta = data.first().unwrap();
        assert_eq!(meta.namespace, namespace);
        assert_eq!(meta.shard_name, shard_name);
        assert_eq!(meta.segment_seq, 0);
        assert_eq!(meta.start_offset, 0);
        assert_eq!(meta.end_offset, -1);
        assert_eq!(meta.start_timestamp, 0);
        assert_eq!(meta.end_timestamp, -1);

        // create next segment
        let request = CreateNextSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        create_next_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        // list segment
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 2);
        let segment = data.first().unwrap();
        assert_eq!(segment.namespace, namespace);
        assert_eq!(segment.shard_name, shard_name);
        assert_eq!(segment.segment_seq, 0);
        assert_eq!(segment.leader_epoch, 0);
        assert_eq!(segment.leader, node_id);
        assert_eq!(segment.status, SegmentStatus::Write);
        assert_eq!(segment.config.max_segment_size, 1073741824);
        assert_eq!(segment.replicas.len(), 1);
        let rep = segment.replicas.first().unwrap();
        assert_eq!(rep.replica_seq, 0);
        assert_eq!(rep.node_id, node_id);
        assert_eq!(rep.fold, node_fold);

        let segment = data.last().unwrap();
        assert_eq!(segment.namespace, namespace);
        assert_eq!(segment.shard_name, shard_name);
        assert_eq!(segment.segment_seq, 1);
        assert_eq!(segment.leader_epoch, 0);
        assert_eq!(segment.leader, node_id);
        assert_eq!(segment.status, SegmentStatus::Idle);
        assert_eq!(segment.config.max_segment_size, 1073741824);
        assert_eq!(segment.replicas.len(), 1);
        let rep = segment.replicas.first().unwrap();
        assert_eq!(rep.replica_seq, 0);
        assert_eq!(rep.node_id, node_id);
        assert_eq!(rep.fold, node_fold);

        // list segment meta
        let request = ListSegmentMetaRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment_meta(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        let data: Vec<JournalSegmentMetadata> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 2);
        let meta = data.first().unwrap();
        assert_eq!(meta.namespace, namespace);
        assert_eq!(meta.shard_name, shard_name);
        assert_eq!(meta.segment_seq, 0);
        assert_eq!(meta.start_offset, 0);
        assert_eq!(meta.end_offset, -1);
        assert_eq!(meta.start_timestamp, 0);
        assert_eq!(meta.end_timestamp, -1);

        let meta = data.last().unwrap();
        assert_eq!(meta.namespace, namespace);
        assert_eq!(meta.shard_name, shard_name);
        assert_eq!(meta.segment_seq, 1);
        assert_eq!(meta.start_offset, -1);
        assert_eq!(meta.end_offset, -1);
        assert_eq!(meta.start_timestamp, -1);
        assert_eq!(meta.end_timestamp, -1);

        // create next segment
        let request = CreateNextSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        create_next_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 2);

        // list segment meta
    }

    #[tokio::test]

    pub async fn sealup_segment_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster_name = unique_id();
        let shard_name = "s1".to_string();
        let namespace = "n1".to_string();
        let node_id = 1;

        // register_node
        let node_fold = "/data1/".to_string();
        let extend = JournalNodeExtend {
            data_fold: vec![node_fold.clone()],
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
        if let Err(e) = register_node(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // create shard
        let request = CreateShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            replica: 1,
        };
        if let Err(e) = create_shard(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // update status to PreSealUp
        let request = UpdateSegmentStatusRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_seq: 0,
            cur_status: "Write".to_string(),
            next_status: "PreSealUp".to_string(),
        };
        if let Err(e) = update_segment_status(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }
        // List Segment
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: 0,
        };
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let segment = data.first().unwrap();
        assert_eq!(segment.status, SegmentStatus::PreSealUp);

        // update status to SealUp
        let request = UpdateSegmentStatusRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_seq: 0,
            cur_status: "PreSealUp".to_string(),
            next_status: "SealUp".to_string(),
        };
        if let Err(e) = update_segment_status(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // List Segment
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: 0,
        };
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let segment = data.first().unwrap();
        assert_eq!(segment.status, SegmentStatus::SealUp);

        // update segment meta
        let request = UpdateSegmentMetaRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: 0,
            start_offset: -1,
            end_offset: 1000,
            start_timestamp: -1,
            end_timestamp: 1731576014,
        };
        if let Err(e) = update_segment_meta(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // list segment meta
        let request = ListSegmentMetaRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: -1,
        };
        let reply = list_segment_meta(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        let data: Vec<JournalSegmentMetadata> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let meta = data.first().unwrap();
        assert_eq!(meta.namespace, namespace);
        assert_eq!(meta.shard_name, shard_name);
        assert_eq!(meta.segment_seq, 0);
        assert_eq!(meta.start_offset, 0);
        assert_eq!(meta.end_offset, 1000);
        assert_eq!(meta.start_timestamp, 0);
        assert_eq!(meta.end_timestamp, 1731576014);
    }

    #[tokio::test]

    pub async fn delete_segment_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster_name = unique_id();
        let shard_name = "s1".to_string();
        let namespace = "n1".to_string();
        let node_id = 1;

        // register_node
        let node_fold = "/data1/".to_string();
        let extend = JournalNodeExtend {
            data_fold: vec![node_fold.clone()],
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
        if let Err(e) = register_node(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            unreachable!();
        }

        // create shard
        let request = CreateShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            replica: 1,
        };
        if let Err(e) = create_shard(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            unreachable!();
        }

        // create next segment
        let request = CreateNextSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        create_next_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        // delete segment
        let request = DeleteSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_seq: 0,
        };
        if let Err(e) = delete_segment(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
        } else {
            unreachable!();
        }

        // update status to SealUp
        let request = UpdateSegmentStatusRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_seq: 0,
            cur_status: "Write".to_string(),
            next_status: "SealUp".to_string(),
        };
        if let Err(e) = update_segment_status(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // delete segment
        let request = DeleteSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_seq: 0,
        };
        delete_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        // List Segment
        let request = ListSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            segment_no: 0,
        };
        let reply = list_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        let data: Vec<JournalSegment> = serde_json::from_slice(&reply.segments).unwrap();
        assert_eq!(data.len(), 1);
        let segment = data.first().unwrap();
        assert_eq!(segment.status, SegmentStatus::PreDelete);
    }

    #[tokio::test]

    pub async fn delete_shard_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let cluster_name = unique_id();
        let shard_name = "s1".to_string();
        let namespace = "n1".to_string();
        let node_id = 1;

        // register_node
        let node_fold = "/data1/".to_string();
        let extend = JournalNodeExtend {
            data_fold: vec![node_fold.clone()],
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
        if let Err(e) = register_node(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // create shard
        let request = CreateShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
            replica: 1,
        };
        if let Err(e) = create_shard(client_pool.clone(), &addrs, request).await {
            println!("{}", e);
            panic!();
        }

        // create next segment
        let request = CreateNextSegmentRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        create_next_segment(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        // delete shard
        let request = DeleteShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        delete_shard(client_pool.clone(), &addrs, request)
            .await
            .unwrap();

        // list shard
        let request = ListShardRequest {
            cluster_name: cluster_name.clone(),
            shard_name: shard_name.clone(),
            namespace: namespace.clone(),
        };
        let reply = list_shard(client_pool.clone(), &addrs, request)
            .await
            .unwrap();
        let data: Vec<JournalShard> = serde_json::from_slice(&reply.shards).unwrap();
        assert_eq!(data.len(), 1);
        let shard_raw = data.first().unwrap();
        assert_eq!(shard_raw.status, JournalShardStatus::PrepareDelete);
    }
}
