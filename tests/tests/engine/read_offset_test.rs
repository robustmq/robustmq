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

    const WRITE_COUNT: usize = 5;

    async fn write_records(conn: &Arc<ClientConnectionManager>, shard_name: &str) -> Vec<u64> {
        let mut messages = Vec::with_capacity(WRITE_COUNT);
        for i in 0..WRITE_COUNT {
            let record = AdapterWriteRecord::new("", Bytes::from(format!("data-{}", i)));
            messages.push(serialize::serialize(&record).unwrap());
        }
        let rows = conn
            .send_write(ENGINE_NODE_ID, shard_name, messages)
            .await
            .expect("send_write failed");
        assert_eq!(rows.len(), WRITE_COUNT);
        rows.iter().map(|r| r.offset).collect()
    }

    async fn read_by_offset(
        conn: &Arc<ClientConnectionManager>,
        shard_name: &str,
        offset: u64,
        max_record: u64,
    ) -> Vec<StorageRecord> {
        let req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.to_string(),
            ReadType::Offset,
            false,
            ReadReqFilter::by_offset(offset),
            ReadReqOptions::new(1024 * 1024, max_record),
        )]));
        conn.send_read(ENGINE_NODE_ID, req)
            .await
            .expect("send_read failed")
    }

    #[tokio::test]
    async fn read_offset_by_segment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#;
        run_read_offset_test(config).await;
    }

    #[tokio::test]
    async fn read_offset_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        run_read_offset_test(config).await;
    }

    #[tokio::test]
    async fn read_offset_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        run_read_offset_test(config).await;
    }

    async fn run_read_offset_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;

        // EngineSegment 的 WriteResp 中 messages 顺序由 HashMap 迭代决定，不保证与写入顺序一致。
        // 排序后验证连续性，取 base 作为后续 read 的起始 offset。
        let mut offsets = write_records(&conn, &shard_name).await;
        offsets.sort();

        assert_eq!(offsets.len(), WRITE_COUNT);
        let base = offsets[0];
        for (i, &off) in offsets.iter().enumerate() {
            assert_eq!(off, base + i as u64);
        }

        // read from base → all WRITE_COUNT records
        let records = read_by_offset(&conn, &shard_name, base, WRITE_COUNT as u64).await;
        assert_eq!(
            records.len(),
            WRITE_COUNT,
            "read from base should return all records"
        );
        for (i, rec) in records.iter().enumerate() {
            assert_eq!(rec.metadata.offset, base + i as u64);
        }

        // read from base+2 → 3 records
        let records = read_by_offset(&conn, &shard_name, base + 2, WRITE_COUNT as u64).await;
        assert_eq!(records.len(), 3, "read from base+2 should return 3 records");
        assert_eq!(records[0].metadata.offset, base + 2);
        assert_eq!(records[1].metadata.offset, base + 3);
        assert_eq!(records[2].metadata.offset, base + 4);

        // max_record=2 from base → only 2 records
        let records = read_by_offset(&conn, &shard_name, base, 2).await;
        assert_eq!(records.len(), 2, "max_record=2 should limit to 2");
        assert_eq!(records[0].metadata.offset, base);
        assert_eq!(records[1].metadata.offset, base + 1);
    }
}
