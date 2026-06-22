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
        admin_client, create_shard, engine_client, read_messages_raw, write_messages,
    };
    use bytes::Bytes;
    use common_base::tools::now_second;
    use common_base::utils::serialize::{self, deserialize};
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::StorageRecord;
    use protocol::storage::protocol::{ReadReqFilter, ReadType};

    #[tokio::test]
    async fn message_ttl_test_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        message_ttl_test(config).await;
    }

    #[tokio::test]
    async fn message_ttl_test_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        message_ttl_test(config).await;
    }

    #[tokio::test]
    async fn message_ttl_test_by_filesegment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#;
        message_ttl_test(config).await;
    }

    async fn message_ttl_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;

        let now = now_second();
        let messages = vec![
            // not expired (expires 5 minutes from now)
            serialize::serialize(
                &AdapterWriteRecord::new("", Bytes::from("alive")).with_expire_at(now + 300),
            )
            .unwrap(),
            // already expired (1 second ago)
            serialize::serialize(
                &AdapterWriteRecord::new("", Bytes::from("expired")).with_expire_at(now - 1),
            )
            .unwrap(),
        ];

        write_messages(&conn, &shard_name, messages).await;

        // Read from offset 0 — only the non-expired record should be returned.
        let raw = read_messages_raw(
            &conn,
            &shard_name,
            ReadType::Offset,
            ReadReqFilter::by_offset(0),
            10,
        )
        .await;

        assert_eq!(
            raw.len(),
            1,
            "expected 1 non-expired message, got {}",
            raw.len()
        );
        let record: StorageRecord = deserialize(&raw[0]).unwrap();
        assert_eq!(record.data, Bytes::from("alive"));
    }
}
