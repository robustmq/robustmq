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

    use crate::engine::common::{admin_client, create_shard, engine_client, ENGINE_NODE_ID};
    use admin_server::engine::record::RecordDeleteByKeysReq;
    use admin_server::engine::segment::{SegmentDetailReq, SegmentDetailResp};
    use bytes::Bytes;
    use common_base::http_response::AdminServerResponse;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::StorageRecord;
    use protocol::storage::protocol::{
        ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use storage_engine::clients::manager::ClientConnectionManager;

    const WRITE_COUNT: usize = 5;

    async fn write_records(conn: &Arc<ClientConnectionManager>, shard_name: &str) -> Vec<u64> {
        let mut messages = Vec::with_capacity(WRITE_COUNT);
        for i in 0..WRITE_COUNT {
            let record = AdapterWriteRecord::new("", Bytes::from(format!("data-{}", i)))
                .with_key(format!("key-{}", i));
            messages.push(serialize::serialize(&record).unwrap());
        }
        let rows = conn
            .send_write(ENGINE_NODE_ID, shard_name, messages)
            .await
            .expect("send_write failed");
        assert_eq!(rows.len(), WRITE_COUNT);
        rows.iter().map(|r| r.offset).collect()
    }

    async fn read_by_key(
        conn: &Arc<ClientConnectionManager>,
        shard_name: &str,
        key: &str,
    ) -> Vec<StorageRecord> {
        let req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.to_string(),
            ReadType::Key,
            false,
            ReadReqFilter::by_key(key.to_string()),
            ReadReqOptions::new(1024 * 1024, 100),
        )]));
        conn.send_read(ENGINE_NODE_ID, req)
            .await
            .expect("send_read failed")
    }

    async fn get_segment0_offsets(
        client: &admin_server::client::AdminHttpClient,
        shard_name: &str,
    ) -> (u64, u64, u64) {
        let raw = client
            .get_segment_detail::<_, AdminServerResponse<SegmentDetailResp>>(&SegmentDetailReq {
                shard_name: shard_name.to_string(),
                segment_seq: 0,
            })
            .await
            .expect("get_segment_detail failed");
        let r = raw
            .data
            .replicas
            .into_iter()
            .next()
            .expect("no replica in segment detail");
        (r.leo, r.high_watermark, r.log_start_offset)
    }

    // EngineSegment does not support delete_by_key; only test write + read.
    #[tokio::test]
    async fn read_key_by_segment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#;
        run_write_and_read_key_test(config).await;
    }

    #[tokio::test]
    async fn read_key_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        run_write_read_delete_key_test(config).await;
    }

    #[tokio::test]
    async fn read_key_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        run_write_read_delete_key_test(config).await;
    }

    async fn run_write_and_read_key_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;
        write_records(&conn, &shard_name).await;

        let (leo, hw, lso) = get_segment0_offsets(&admin, &shard_name).await;
        assert_eq!(leo, WRITE_COUNT as u64);
        assert_eq!(hw, WRITE_COUNT as u64);
        assert_eq!(lso, 0);

        let records = read_by_key(&conn, &shard_name, "key-2").await;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].metadata.key.as_deref(), Some(b"key-2".as_ref()));

        let records = read_by_key(&conn, &shard_name, "key-999").await;
        assert_eq!(records.len(), 0);
    }

    async fn run_write_read_delete_key_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;
        write_records(&conn, &shard_name).await;

        let (leo, hw, lso) = get_segment0_offsets(&admin, &shard_name).await;
        assert_eq!(leo, WRITE_COUNT as u64);
        assert_eq!(hw, WRITE_COUNT as u64);
        assert_eq!(lso, 0);

        let records = read_by_key(&conn, &shard_name, "key-3").await;
        assert_eq!(records.len(), 1, "key-3 should exist before delete");
        assert_eq!(records[0].metadata.key.as_deref(), Some(b"key-3".as_ref()));

        let resp = admin
            .delete_record_by_keys(&RecordDeleteByKeysReq {
                shard_name: shard_name.clone(),
                keys: vec!["key-3".to_string()],
            })
            .await
            .expect("delete_record_by_keys http failed");
        let v: AdminServerResponse<serde_json::Value> = serde_json::from_str(&resp).unwrap();
        assert_eq!(v.code, 0, "delete_record_by_keys failed: {:?}", v.error);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
        let records = loop {
            let r = read_by_key(&conn, &shard_name, "key-3").await;
            if r.is_empty() || tokio::time::Instant::now() >= deadline {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        };
        assert_eq!(records.len(), 0, "key-3 should be gone after delete");

        for i in 0..WRITE_COUNT {
            if i == 3 {
                continue;
            }
            let records = read_by_key(&conn, &shard_name, &format!("key-{}", i)).await;
            assert_eq!(records.len(), 1, "key-{} should still exist", i);
        }
    }
}
