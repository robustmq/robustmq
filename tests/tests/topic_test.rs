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
    use admin_server::client::AdminHttpClient;
    use admin_server::cluster::topic::{TopicCreateReq, TopicDeleteRep, TopicListReq};
    use admin_server::engine::segment::{SegmentListReq, SegmentListResp};
    use admin_server::engine::shard::{ShardListReq, ShardListRow};
    use common_base::http_response::AdminServerResponse;
    use common_base::uuid::unique_id;
    use common_config::storage::StorageType;
    use metadata_struct::adapter::adapter_shard::AdapterShardDetailExtend;
    use metadata_struct::storage::segment::SegmentStatus;
    use metadata_struct::storage::shard::{
        EngineShardStatus, DEFAULT_MAX_SEGMENT_SIZE, DEFAULT_RETENTION_SEC,
    };
    use metadata_struct::topic::Topic;
    use std::time::Duration;
    use tokio::time::sleep;

    pub async fn create_test_env() -> AdminHttpClient {
        AdminHttpClient::new("http://127.0.0.1:8080")
    }

    #[tokio::test]
    async fn topic_test() {
        let client = create_test_env().await;
        let tenant = "public".to_string();
        let topic_name = format!("test-topic-{}", unique_id());
        let partition = 1u32;
        let replication = 1u32;

        // ── create topic — verify returned metadata ───────────────────────────
        let create_req = TopicCreateReq {
            tenant: tenant.clone(),
            topic_name: topic_name.clone(),
            storage_type: "EngineMemory".to_string(),
            source: "MQTT".to_string(),
            partition: Some(partition),
            replication: Some(replication),
        };
        let topic: Topic = client.create_topic(&create_req).await.unwrap();
        println!("create_topic result: {:#?}", topic);

        assert_eq!(topic.topic_name, topic_name);
        assert_eq!(topic.tenant, tenant);
        assert_eq!(topic.partition, partition);
        assert_eq!(topic.replication, replication);
        assert_eq!(topic.storage_type, StorageType::EngineMemory);
        assert!(!topic.mark_delete);
        assert!(!topic.topic_id.is_empty(), "topic_id should be generated");
        assert!(topic.create_time > 0);
        // storage_name_list must have one entry per partition
        assert_eq!(
            topic.storage_name_list.len(),
            partition as usize,
            "storage_name_list length should equal partition count"
        );

        // build list_req for later cleanup checks
        let list_req = TopicListReq {
            tenant: Some(tenant.clone()),
            topic_name: Some(topic_name.clone()),
            ..Default::default()
        };

        // ── list shard info — one shard per partition, verify metadata ────────
        // partition=1 means storage_name_list has key 0
        let shard_name = topic.storage_name_list[&0].clone();
        let shard_req = ShardListReq {
            shard_name: Some(shard_name.clone()),
            ..Default::default()
        };
        let shard_list = client
            .get_shard_list::<_, Vec<ShardListRow>>(&shard_req)
            .await
            .unwrap();
        println!("shard list after create: {:#?}", shard_list);

        assert_eq!(shard_list.data.len(), 1, "expected exactly 1 shard");
        let shard_row = &shard_list.data[0];
        let shard_detail = &shard_row.shard_info;

        // shard name matches
        assert_eq!(shard_detail.shard_name, shard_name);

        // config: replica_num, storage_type, max_segment_size, retention_sec
        assert_eq!(shard_detail.config.replica_num, replication);
        assert_eq!(shard_detail.config.storage_type, StorageType::EngineMemory);
        assert_eq!(
            shard_detail.config.max_segment_size,
            Some(DEFAULT_MAX_SEGMENT_SIZE)
        );
        assert_eq!(shard_detail.config.retention_sec, DEFAULT_RETENTION_SEC);

        // extend: StorageEngine variant, initial seq values, status=Run
        let AdapterShardDetailExtend::StorageEngine(engine_shard) = &shard_detail.extend;
        assert_eq!(engine_shard.start_segment_seq, 0);
        assert_eq!(engine_shard.active_segment_seq, 0);
        assert_eq!(engine_shard.last_segment_seq, 0);
        assert_eq!(engine_shard.status, EngineShardStatus::Run);

        // ── list segment info — one segment (seq=0) per shard ─────────────────
        let segment_req = SegmentListReq {
            shard_name: shard_name.clone(),
        };
        let segment_resp_str = client.get_segment_list(&segment_req).await.unwrap();
        println!("segment list after create: {}", segment_resp_str);

        let segment_data: AdminServerResponse<SegmentListResp> =
            serde_json::from_str(&segment_resp_str).unwrap();
        assert_eq!(segment_data.code, 0);
        let segment_resp = segment_data.data;

        assert_eq!(
            segment_resp.segment_list.len(),
            1,
            "expected exactly 1 segment"
        );
        let seg = &segment_resp.segment_list[0];
        assert_eq!(seg.segment.shard_name, shard_name);
        assert_eq!(
            seg.segment.segment_seq, 0,
            "initial segment seq should be 0"
        );
        assert_eq!(
            seg.segment.status,
            SegmentStatus::Write,
            "initial segment should be in Write status"
        );

        // ── delete topic ──────────────────────────────────────────────────────
        let delete_req = TopicDeleteRep {
            tenant: tenant.clone(),
            topic_name: topic_name.clone(),
        };
        let delete_result = client.delete_topic(&delete_req).await.unwrap();
        println!("delete_topic result: {}", delete_result);

        // ── sleep 10s — wait for async shard/segment cleanup ──────────────────
        sleep(Duration::from_secs(10)).await;

        // ── list topic — verify topic is gone ─────────────────────────────────
        let topic_list_final = client
            .get_topic_list::<_, Vec<Topic>>(&list_req)
            .await
            .unwrap();
        println!("topic list after cleanup: {:#?}", topic_list_final);
        assert_eq!(
            topic_list_final.data.len(),
            0,
            "topic should be removed after delete"
        );

        // ── list shard — verify shard is cleaned up ───────────────────────────
        let shard_list_final = client
            .get_shard_list::<_, Vec<ShardListRow>>(&shard_req)
            .await
            .unwrap();
        println!("shard list after cleanup: {:#?}", shard_list_final);
        assert_eq!(
            shard_list_final.data.len(),
            0,
            "shard should be removed after topic delete"
        );

        // ── list segment — verify segments are cleaned up ─────────────────────
        let segment_resp_str_final = client.get_segment_list(&segment_req).await.unwrap();
        println!("segment list after cleanup: {}", segment_resp_str_final);
        let segment_data_final: AdminServerResponse<SegmentListResp> =
            serde_json::from_str(&segment_resp_str_final).unwrap();
        assert_eq!(segment_data_final.code, 0);
        assert_eq!(
            segment_data_final.data.segment_list.len(),
            0,
            "segments should be removed after topic delete"
        );
    }
}
