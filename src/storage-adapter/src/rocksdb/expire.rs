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

use crate::message_expire::MessageExpireConfig;
use crate::rocksdb::key::*;
use crate::storage::ShardInfo;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use metadata_struct::adapter::record::Record;
use rocksdb::WriteBatch;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Execute message expiration based on timestamp
///
/// # Arguments
/// * `db` - RocksDB engine instance
/// * `config` - Message expiration configuration containing retention period
///
/// # Logic
/// Delete messages where: message.timestamp < (current_time - retention_period)
pub async fn expire_messages_by_timestamp(
    db: Arc<RocksDBEngine>,
    config: &MessageExpireConfig,
) -> Result<(), CommonError> {
    let retention_seconds = match config.get_timestamp() {
        Some(ts) => ts as u64,
        None => {
            debug!("No timestamp configured for message expiration, skipping");
            return Ok(());
        }
    };

    let current_time = now_second();
    let expire_before = current_time.saturating_sub(retention_seconds);

    debug!(
        "Starting message expiration: current_time={}, retention={}s, expire_before={}",
        current_time, retention_seconds, expire_before
    );

    let cf = db.cf_handle(DB_COLUMN_FAMILY_BROKER).ok_or_else(|| {
        CommonError::CommonError(format!(
            "Column family '{}' not found",
            DB_COLUMN_FAMILY_BROKER
        ))
    })?;

    // Step 1: Get all shards
    let shard_prefix = "/shard/";
    let shard_entries = db.read_prefix(cf.clone(), shard_prefix)?;

    let mut total_deleted = 0_usize;
    let mut total_scanned = 0_usize;

    // Step 2: Process each shard
    for (shard_key, shard_data) in shard_entries {
        let shard_info: ShardInfo = match bincode::deserialize(&shard_data) {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "Failed to deserialize shard info for key {}: {}",
                    shard_key, e
                );
                continue;
            }
        };

        let namespace = &shard_info.namespace;
        let shard_name = &shard_info.shard_name;

        debug!(
            "Processing shard: namespace={}, shard_name={}",
            namespace, shard_name
        );

        // Step 3: Scan records in this shard
        let record_prefix = shard_record_key_prefix(namespace, shard_name);
        let records = db.read_prefix(cf.clone(), &record_prefix)?;

        let mut batch = WriteBatch::default();
        let mut deleted_count = 0_usize;

        for (record_key, record_data) in records {
            total_scanned += 1;

            // Deserialize record to check timestamp
            let record: Record = match bincode::deserialize(&record_data) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to deserialize record {}: {}", record_key, e);
                    continue;
                }
            };

            // Check if record should be expired
            if record.timestamp > 0 && record.timestamp < expire_before {
                // Delete the record
                batch.delete_cf(&cf, record_key.as_bytes());
                deleted_count += 1;

                // Also delete associated indexes
                let offset = record.offset.unwrap_or(0);

                // Delete key index
                if !record.key.is_empty() {
                    let key_index = key_offset_key(namespace, shard_name, &record.key);
                    batch.delete_cf(&cf, key_index.as_bytes());
                }

                // Delete tag indexes
                for tag in &record.tags {
                    let tag_index = tag_offsets_key(namespace, shard_name, tag, offset);
                    batch.delete_cf(&cf, tag_index.as_bytes());
                }

                // Delete timestamp index (if exists)
                if record.timestamp > 0 && offset.is_multiple_of(5000) {
                    let ts_index =
                        timestamp_offset_key(namespace, shard_name, record.timestamp, offset);
                    batch.delete_cf(&cf, ts_index.as_bytes());
                }
            }

            // Batch commit every 1000 deletions to avoid too large batch
            if deleted_count > 0 && deleted_count.is_multiple_of(1000) {
                db.write_batch(batch)?;
                batch = WriteBatch::default();
                debug!("Committed batch deletion: {} records", deleted_count);
            }
        }

        // Commit remaining deletions
        if deleted_count > 0 {
            db.write_batch(batch)?;
            info!(
                "Expired {} messages from shard {}/{}",
                deleted_count, namespace, shard_name
            );
            total_deleted += deleted_count;
        }
    }

    info!(
        "Message expiration completed: scanned={}, deleted={}",
        total_scanned, total_deleted
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rocksdb::RocksDBStorageAdapter;
    use crate::storage::{ShardInfo, StorageAdapter};
    use common_base::tools::unique_id;
    use metadata_struct::adapter::record::Record;
    use rocksdb_engine::test::test_rocksdb_instance;

    #[tokio::test]
    async fn test_expire_messages_by_timestamp() {
        let db = test_rocksdb_instance();
        let adapter = RocksDBStorageAdapter::new(db.clone());

        let namespace = unique_id();
        let shard_name = "test-shard".to_string();

        // Create shard
        adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let now = now_second();

        // Write messages with different timestamps
        // All messages are recent so they won't be expired yet
        let messages = vec![
            Record {
                offset: None,
                header: vec![],
                key: "key1".to_string(),
                data: b"data1".to_vec(),
                tags: vec!["tag1".to_string()],
                timestamp: now - 3600, // 1 hour ago (should be expired with 30min retention)
                crc_num: 0,
            },
            Record {
                offset: None,
                header: vec![],
                key: "key2".to_string(),
                data: b"data2".to_vec(),
                tags: vec!["tag2".to_string()],
                timestamp: now - 100, // Recent (should be kept)
                crc_num: 0,
            },
            Record {
                offset: None,
                header: vec![],
                key: "key3".to_string(),
                data: b"data3".to_vec(),
                tags: vec!["tag3".to_string()],
                timestamp: now - 50, // Recent (should be kept)
                crc_num: 0,
            },
        ];

        adapter
            .batch_write(&namespace, &shard_name, &messages)
            .await
            .unwrap();

        // Verify all messages exist
        let read_config = metadata_struct::adapter::read_config::ReadConfig {
            max_record_num: 10,
            max_size: u64::MAX,
        };
        let records = adapter
            .read_by_offset(&namespace, &shard_name, 0, &read_config)
            .await
            .unwrap();
        assert_eq!(records.len(), 3);

        // Expire messages older than 30 minutes (1800 seconds)
        let config = MessageExpireConfig::with_timestamp(1800);

        expire_messages_by_timestamp(db.clone(), &config)
            .await
            .unwrap();

        // Verify: first message (1 hour old) should be expired
        // Note: batch read stops at first missing record, so we can't read offset 1-2 directly
        // Instead, we check specific offsets individually
        let cf = db.cf_handle(DB_COLUMN_FAMILY_BROKER).unwrap();

        // Check offset 0 (should be deleted)
        let key0 = shard_record_key(&namespace, &shard_name, 0);
        let record0 = db.read::<Record>(cf.clone(), &key0).unwrap();
        assert!(record0.is_none(), "Offset 0 should be expired");

        // Check offset 1 (should exist)
        let key1 = shard_record_key(&namespace, &shard_name, 1);
        let record1 = db.read::<Record>(cf.clone(), &key1).unwrap();
        assert!(record1.is_some(), "Offset 1 should exist");
        assert_eq!(record1.unwrap().key, "key2");

        // Check offset 2 (should exist)
        let key2 = shard_record_key(&namespace, &shard_name, 2);
        let record2 = db.read::<Record>(cf.clone(), &key2).unwrap();
        assert!(record2.is_some(), "Offset 2 should exist");
        assert_eq!(record2.unwrap().key, "key3");
    }
}
