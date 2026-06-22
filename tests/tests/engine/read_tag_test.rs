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
    use bytes::Bytes;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::StorageRecord;
    use protocol::storage::protocol::{
        ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use storage_engine::clients::manager::ClientConnectionManager;

    // Write 6 records: indices 0,1,2 → "tag-a"; indices 3,4,5 → "tag-b"
    async fn write_tagged_records(
        conn: &Arc<ClientConnectionManager>,
        shard_name: &str,
    ) -> Vec<u64> {
        let mut messages = Vec::with_capacity(6);
        for i in 0..6usize {
            let tag = if i < 3 { "tag-a" } else { "tag-b" };
            let record = AdapterWriteRecord::new("", Bytes::from(format!("data-{}", i)))
                .with_tags(vec![tag.to_string()]);
            messages.push(serialize::serialize(&record).unwrap());
        }
        let rows = conn
            .send_write(ENGINE_NODE_ID, shard_name, messages)
            .await
            .expect("send_write failed");
        assert_eq!(rows.len(), 6);
        rows.iter().map(|r| r.offset).collect()
    }

    async fn read_by_tag(
        conn: &Arc<ClientConnectionManager>,
        shard_name: &str,
        tag: &str,
    ) -> Vec<StorageRecord> {
        let req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.to_string(),
            ReadType::Tag,
            false,
            ReadReqFilter::by_tag(tag.to_string()),
            ReadReqOptions::new(1024 * 1024, 100),
        )]));
        conn.send_read(ENGINE_NODE_ID, req)
            .await
            .expect("send_read failed")
    }

    #[tokio::test]
    async fn read_tag_by_segment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#;
        run_read_tag_test(config).await;
    }

    #[tokio::test]
    async fn read_tag_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        run_read_tag_test(config).await;
    }

    #[tokio::test]
    async fn read_tag_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        run_read_tag_test(config).await;
    }

    async fn run_read_tag_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;
        let mut offsets = write_tagged_records(&conn, &shard_name).await;
        // response order is non-deterministic for EngineSegment; sort to get contiguous range
        offsets.sort();
        assert_eq!(offsets.len(), 6);
        let base = offsets[0];

        // "tag-a" → first 3 records (base, base+1, base+2)
        let records = read_by_tag(&conn, &shard_name, "tag-a").await;
        assert_eq!(records.len(), 3, "tag-a should return 3 records");
        for rec in &records {
            let tags = rec.metadata.tags.as_deref().unwrap_or(&[]);
            assert!(
                tags.contains(&"tag-a".to_string()),
                "record should have tag-a"
            );
            assert!(
                rec.metadata.offset < base + 3,
                "tag-a records should be at offsets base..base+2, got {}",
                rec.metadata.offset
            );
        }

        // "tag-b" → last 3 records (base+3, base+4, base+5)
        let records = read_by_tag(&conn, &shard_name, "tag-b").await;
        assert_eq!(records.len(), 3, "tag-b should return 3 records");
        for rec in &records {
            let tags = rec.metadata.tags.as_deref().unwrap_or(&[]);
            assert!(
                tags.contains(&"tag-b".to_string()),
                "record should have tag-b"
            );
            assert!(
                rec.metadata.offset >= base + 3,
                "tag-b records should be at offsets base+3..base+5, got {}",
                rec.metadata.offset
            );
        }

        // unknown tag → 0 records
        let records = read_by_tag(&conn, &shard_name, "tag-none").await;
        assert_eq!(records.len(), 0, "unknown tag should return 0 records");
    }
}
