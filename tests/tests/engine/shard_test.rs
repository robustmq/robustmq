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
    use std::time::Duration;

    use crate::engine::common::{
        admin_client, create_shard, get_segment_list, get_shard_list, grpc_pool, meta_addr,
        poll_until,
    };
    use admin_server::engine::shard::ShardDeleteReq;
    use common_base::http_response::AdminServerResponse;
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use common_config::storage::StorageType;
    use grpc_clients::meta::storage::call::{
        create_next_segment, delete_segment, update_start_time_by_segment_meta,
    };
    use metadata_struct::storage::segment::SegmentStatus;
    use metadata_struct::storage::shard::EngineShardStatus;
    use protocol::meta::meta_service_journal::{
        CreateNextSegmentRequest, DeleteSegmentRaw, DeleteSegmentRequest,
        UpdateStartTimeBySegmentMetaRequest,
    };

    const SHARD_CONFIG: &str = r#"{
        "replica_num": 1,
        "max_segment_size": 1073741824,
        "retention_sec": 86400,
        "storage_type": "EngineSegment"
    }"#;

    #[tokio::test]
    async fn shard_lifecycle() {
        let client = admin_client();
        let shard_name = unique_id();

        create_shard(&client, &shard_name, SHARD_CONFIG).await;

        let shards = get_shard_list(&client, &shard_name).await;
        assert_eq!(shards.len(), 1, "expected exactly one shard after create");

        let row = &shards[0].shard_info;
        assert_eq!(row.shard_name, shard_name);
        assert_eq!(row.config.replica_num, 1);
        assert_eq!(row.config.max_segment_size, Some(1073741824));
        assert_eq!(row.config.retention_sec, 86400);
        assert_eq!(row.config.storage_type, StorageType::EngineSegment);

        let shard = &row.shard;
        assert!(!shard.shard_uid.is_empty(), "shard_uid must not be empty");
        assert_eq!(shard.shard_name, shard_name);
        assert_eq!(shard.status, EngineShardStatus::Run);
        assert_eq!(shard.start_segment_seq, 0);
        assert_eq!(shard.active_segment_seq, 0);
        assert_eq!(shard.last_segment_seq, 0);

        let segments = get_segment_list(&client, &shard_name).await;
        assert_eq!(segments.len(), 1, "expected segment-0 auto-created");

        let seg = &segments[0].segment;
        assert_eq!(seg.segment_seq, 0);
        assert_eq!(seg.shard_name, shard_name);
        assert_eq!(seg.status, SegmentStatus::Write);
        assert!(
            !seg.replicas.is_empty(),
            "segment must have at least one replica"
        );

        let meta = segments[0]
            .segment_meta
            .as_ref()
            .expect("segment-0 must have metadata");
        assert_eq!(meta.segment_seq, 0);
        assert_eq!(meta.start_offset, 0);
        assert_eq!(meta.end_offset, 0);
        assert_eq!(meta.start_timestamp, -1);
        assert_eq!(meta.end_timestamp, -1);

        let resp = client
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await
            .expect("delete_shard http failed");
        let resp_val: AdminServerResponse<serde_json::Value> =
            serde_json::from_str(&resp).expect("parse delete response");
        assert_eq!(
            resp_val.code, 0,
            "delete_shard failed: {:?}",
            resp_val.error
        );

        let shard_gone = poll_until(Duration::from_secs(30), Duration::from_secs(2), || async {
            get_shard_list(&client, &shard_name).await.is_empty()
        })
        .await;
        assert!(shard_gone, "shard still present after 30s");

        let segments_gone = poll_until(Duration::from_secs(30), Duration::from_secs(2), || async {
            get_segment_list(&client, &shard_name).await.is_empty()
        })
        .await;
        assert!(segments_gone, "segments still present after 30s");
    }

    #[tokio::test]
    async fn segment_lifecycle() {
        let client = admin_client();
        let pool = grpc_pool();
        let shard_name = unique_id();

        create_shard(&client, &shard_name, SHARD_CONFIG).await;

        let seg0_ready = poll_until(
            Duration::from_secs(10),
            Duration::from_millis(500),
            || async { !get_segment_list(&client, &shard_name).await.is_empty() },
        )
        .await;
        assert!(seg0_ready, "segment-0 did not appear within 10s");

        let segments = get_segment_list(&client, &shard_name).await;
        assert_eq!(segments.len(), 1);
        let seg0 = &segments[0];
        assert_eq!(seg0.segment.segment_seq, 0);
        assert_eq!(seg0.segment.status, SegmentStatus::Write);
        let meta0 = seg0.segment_meta.as_ref().expect("seg-0 must have meta");
        assert_eq!(meta0.start_offset, 0);
        assert_eq!(meta0.end_offset, 0);
        assert_eq!(meta0.start_timestamp, -1);
        assert_eq!(meta0.end_timestamp, -1);

        let t_start = now_second();
        update_start_time_by_segment_meta(
            &pool,
            &[meta_addr()],
            UpdateStartTimeBySegmentMetaRequest {
                shard_name: shard_name.clone(),
                segment: 0,
                start_timestamp: t_start,
            },
        )
        .await
        .expect("update_start_time failed");

        const END_OFFSET: i64 = 99;
        create_next_segment(
            &pool,
            &[meta_addr()],
            CreateNextSegmentRequest {
                shard_name: shard_name.clone(),
                current_segment: 0,
                current_segment_end_offset: END_OFFSET,
            },
        )
        .await
        .expect("create_next_segment failed");

        let seg1_ready = poll_until(
            Duration::from_secs(10),
            Duration::from_millis(500),
            || async { get_segment_list(&client, &shard_name).await.len() >= 2 },
        )
        .await;
        assert!(seg1_ready, "segment-1 did not appear within 10s");

        let segments = get_segment_list(&client, &shard_name).await;
        assert_eq!(segments.len(), 2);

        let seg0 = segments
            .iter()
            .find(|r| r.segment.segment_seq == 0)
            .expect("segment-0 not found");
        assert_eq!(seg0.segment.status, SegmentStatus::SealUp);
        let meta0 = seg0.segment_meta.as_ref().expect("seg-0 must have meta");
        assert_eq!(meta0.start_offset, 0);
        assert_eq!(meta0.end_offset, END_OFFSET);
        assert_eq!(meta0.start_timestamp, t_start as i64);
        assert!(meta0.end_timestamp > 0, "seg-0 end_timestamp must be set");

        let seg1 = segments
            .iter()
            .find(|r| r.segment.segment_seq == 1)
            .expect("segment-1 not found");
        assert_eq!(seg1.segment.status, SegmentStatus::Write);
        assert_eq!(seg1.segment.shard_name, shard_name);
        assert!(!seg1.segment.replicas.is_empty());
        let meta1 = seg1.segment_meta.as_ref().expect("seg-1 must have meta");
        assert_eq!(meta1.start_offset, END_OFFSET + 1);
        assert_eq!(meta1.end_offset, 0);
        assert_eq!(meta1.start_timestamp, -1);
        assert_eq!(meta1.end_timestamp, -1);

        delete_segment(
            &pool,
            &[meta_addr()],
            DeleteSegmentRequest {
                segment_list: vec![DeleteSegmentRaw {
                    shard_name: shard_name.clone(),
                    segment: 0,
                }],
            },
        )
        .await
        .expect("delete_segment failed");

        let seg0_gone = poll_until(Duration::from_secs(30), Duration::from_secs(2), || async {
            let list = get_segment_list(&client, &shard_name).await;
            list.len() == 1 && list[0].segment.segment_seq == 1
        })
        .await;
        assert!(seg0_gone, "segment-0 still present after 30s");

        let segments = get_segment_list(&client, &shard_name).await;
        assert_eq!(segments.len(), 1);
        let remaining = &segments[0];
        assert_eq!(remaining.segment.segment_seq, 1);
        assert_eq!(remaining.segment.status, SegmentStatus::Write);
        let meta = remaining
            .segment_meta
            .as_ref()
            .expect("seg-1 must have meta");
        assert_eq!(meta.start_offset, END_OFFSET + 1);
        assert_eq!(meta.end_offset, 0);
        assert_eq!(meta.start_timestamp, -1);
        assert_eq!(meta.end_timestamp, -1);
    }
}
