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
    use crate::engine::common::{
        admin_client, create_shard, engine_client, read_messages, write_messages,
    };
    use bytes::Bytes;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use protocol::storage::protocol::{ReadReqFilter, ReadType};

    #[tokio::test]
    async fn key_compact_test_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        key_compact_test(config).await;
    }

    #[tokio::test]
    async fn key_compact_test_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        key_compact_test(config).await;
    }

    #[tokio::test]
    async fn key_compact_test_by_filesegment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#;
        key_compact_test(config).await;
    }

    async fn key_compact_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;

        // Write 3 records with the same key — key compaction should keep only the latest.
        let messages: Vec<Vec<u8>> = (1..=3)
            .map(|i| {
                let record = AdapterWriteRecord::new("", Bytes::from(format!("value-{}", i)))
                    .with_key("status");
                serialize::serialize(&record).unwrap()
            })
            .collect();

        write_messages(&conn, &shard_name, messages).await;

        // Read by key — expect exactly 1 record: the last written value.
        let records = read_messages(
            &conn,
            &shard_name,
            ReadType::Key,
            ReadReqFilter {
                offset: Some(0),
                key: Some("status".to_string().into()),
                ..Default::default()
            },
            10,
        )
        .await;

        assert_eq!(
            records.len(),
            1,
            "key compact should return exactly 1 message"
        );
        assert_eq!(records[0].metadata.key.as_deref(), Some(b"status".as_ref()));
        assert_eq!(records[0].data, Bytes::from("value-3"));
    }
}
