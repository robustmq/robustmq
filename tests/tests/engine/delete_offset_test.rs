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

// EngineSegment does not support delete_by_offset; only EngineMemory and EngineRocksDB are tested.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::engine::common::{admin_client, create_shard, engine_client, ENGINE_NODE_ID};
    use admin_server::engine::record::RecordDeleteByOffsetsReq;
    use bytes::Bytes;
    use common_base::http_response::AdminServerResponse;
    use common_base::utils::serialize::{self, deserialize};
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::StorageRecord;
    use protocol::storage::codec::StorageEnginePacket;
    use protocol::storage::protocol::{
        ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType, WriteReq,
        WriteReqBody,
    };
    use storage_engine::clients::manager::ClientConnectionManager;

    const WRITE_COUNT: usize = 5;

    async fn write_records(conn: &Arc<ClientConnectionManager>, shard_name: &str) -> Vec<u64> {
        let mut messages = Vec::with_capacity(WRITE_COUNT);
        for i in 0..WRITE_COUNT {
            let record = AdapterWriteRecord::new("", Bytes::from(format!("data-{}", i)));
            messages.push(serialize::serialize(&record).unwrap());
        }
        let req = WriteReq::new(WriteReqBody::new(shard_name.to_string(), messages));
        let resp = conn
            .write_send(ENGINE_NODE_ID, StorageEnginePacket::WriteReq(req))
            .await
            .expect("write_send failed");
        match resp {
            StorageEnginePacket::WriteResp(r) => {
                if let Some(err) = r.header.error {
                    panic!("WriteResp error: {}:{}", err.code, err.error);
                }
                let mut offsets: Vec<u64> =
                    r.body.status[0].messages.iter().map(|m| m.offset).collect();
                offsets.sort();
                offsets
            }
            other => panic!("expected WriteResp, got {}", other),
        }
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
        let resp = conn
            .read_send(ENGINE_NODE_ID, StorageEnginePacket::ReadReq(req))
            .await
            .expect("read_send failed");
        match resp {
            StorageEnginePacket::ReadResp(r) => {
                if let Some(err) = r.header.error {
                    panic!("ReadResp(Offset) error: {}:{}", err.code, err.error);
                }
                r.body
                    .messages
                    .iter()
                    .map(|b| deserialize::<StorageRecord>(b).expect("deserialize failed"))
                    .collect()
            }
            other => panic!("expected ReadResp, got {}", other),
        }
    }

    #[tokio::test]
    async fn delete_offset_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        run_delete_offset_test(config).await;
    }

    #[tokio::test]
    async fn delete_offset_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        run_delete_offset_test(config).await;
    }

    async fn run_delete_offset_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;
        let offsets = write_records(&conn, &shard_name).await;
        assert_eq!(offsets.len(), WRITE_COUNT);
        let base = offsets[0];

        // read from base → all 5 records exist
        let records = read_by_offset(&conn, &shard_name, base, WRITE_COUNT as u64).await;
        assert_eq!(
            records.len(),
            WRITE_COUNT,
            "all records should exist before delete"
        );

        // delete offsets base+1 and base+3
        let delete_offsets = vec![base + 1, base + 3];
        let resp = admin
            .delete_record_by_offsets(&RecordDeleteByOffsetsReq {
                shard_name: shard_name.clone(),
                offsets: delete_offsets.clone(),
            })
            .await
            .expect("delete_record_by_offsets http failed");
        let v: AdminServerResponse<serde_json::Value> = serde_json::from_str(&resp).unwrap();
        assert_eq!(v.code, 0, "delete_record_by_offsets failed: {:?}", v.error);

        // read from base → only 3 records remain (base, base+2, base+4)
        let records = read_by_offset(&conn, &shard_name, base, WRITE_COUNT as u64).await;
        assert_eq!(records.len(), 3, "3 records should remain after deleting 2");

        let remaining_offsets: Vec<u64> = records.iter().map(|r| r.metadata.offset).collect();
        for deleted in &delete_offsets {
            assert!(
                !remaining_offsets.contains(deleted),
                "offset {} should be gone after delete",
                deleted
            );
        }
        for kept in [base, base + 2, base + 4] {
            assert!(
                remaining_offsets.contains(&kept),
                "offset {} should still exist",
                kept
            );
        }
    }
}
