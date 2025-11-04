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

use crate::file::key::*;
use crate::storage::ShardInfo;
use crate::{expire::MessageExpireConfig, file::parse_offset_bytes};
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
        None => 3600,
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

    let shard_prefix = "/shard/";
    let shard_entries = db.read_prefix(cf.clone(), shard_prefix)?;

    let mut total_deleted = 0_usize;
    let mut total_scanned = 0_usize;

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

        let record_prefix = shard_record_key_prefix(namespace, shard_name);
        let mut iter = db.db.raw_iterator_cf(&cf);
        iter.seek(&record_prefix);

        let mut batch = WriteBatch::default();
        let mut shard_scanned = 0_usize;

        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let record_key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !record_key.starts_with(&record_prefix) {
                break;
            }

            let Some(record_data) = iter.value() else {
                iter.next();
                continue;
            };

            shard_scanned += 1;
            total_scanned += 1;

            let record: Record = match bincode::deserialize(record_data) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to deserialize record {}: {}", record_key, e);
                    iter.next();
                    continue;
                }
            };

            if record.timestamp > 0 && record.timestamp < expire_before {
                batch.delete_cf(&cf, record_key.as_bytes());

                let offset = record.offset.unwrap_or(0);

                // Delete key index (only if it points to this offset)
                // Key index is a key -> latest_offset mapping
                // We should only delete it if it still points to this expired record
                if let Some(key) = &record.key {
                    let key_index_key = key_offset_key(namespace, shard_name, key);

                    if let Ok(Some(indexed_offset_bytes)) = db.db.get_cf(&cf, &key_index_key) {
                        if let Ok(indexed_offset) = parse_offset_bytes(&indexed_offset_bytes) {
                            // Only delete if it points to the current expired record
                            if indexed_offset == offset {
                                batch.delete_cf(&cf, key_index_key.as_bytes());
                            }
                        }
                    }
                }

                // Delete tag indexes (always safe to delete as tags are many-to-many)
                if let Some(tags) = &record.tags {
                    for tag in tags {
                        let tag_index = tag_offsets_key(namespace, shard_name, tag, offset);
                        batch.delete_cf(&cf, tag_index.as_bytes());
                    }
                }

                if record.timestamp > 0 && offset.is_multiple_of(5000) {
                    let ts_index =
                        timestamp_offset_key(namespace, shard_name, record.timestamp, offset);
                    batch.delete_cf(&cf, ts_index.as_bytes());
                }
            }
            iter.next();
        }

        let deleted_count = batch.len();
        db.write_batch(batch)?;

        if deleted_count > 0 {
            info!(
                "Expired {} messages from shard {}/{} (scanned {})",
                deleted_count, namespace, shard_name, shard_scanned
            );
            total_deleted += deleted_count;
        }
    }

    debug!(
        "Message expiration completed: scanned={}, deleted={}",
        total_scanned, total_deleted
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::RocksDBStorageAdapter;
    use crate::storage::{ShardInfo, StorageAdapter};
    use common_base::tools::unique_id;
    use metadata_struct::adapter::record::Record;
    use rocksdb_engine::test::test_storage_driver_rockdb_config;

    #[tokio::test]
    async fn test_expire_messages_by_timestamp() {
        let adapter = RocksDBStorageAdapter::new(test_storage_driver_rockdb_config());

        let namespace = unique_id();
        let shard_name = "test-shard".to_string();

        adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let now = now_second();

        let messages = vec![
            Record {
                offset: None,
                header: None,
                key: Some("key1".to_string()),
                data: b"data1".to_vec().into(),
                tags: Some(vec!["tag1".to_string()]),
                timestamp: now - 3600, // 1 hour ago (should be expired with 30min retention)
                crc_num: 0,
            },
            Record {
                offset: None,
                header: None,
                key: Some("key2".to_string()),
                data: b"data2".to_vec().into(),
                tags: Some(vec!["tag2".to_string()]),
                timestamp: now - 100, // Recent (should be kept)
                crc_num: 0,
            },
            Record {
                offset: None,
                header: None,
                key: Some("key3".to_string()),
                data: b"data3".to_vec().into(),
                tags: Some(vec!["tag3".to_string()]),
                timestamp: now - 50, // Recent (should be kept)
                crc_num: 0,
            },
        ];

        adapter
            .batch_write(&namespace, &shard_name, &messages)
            .await
            .unwrap();

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

        expire_messages_by_timestamp(adapter.db.clone(), &config)
            .await
            .unwrap();

        // Verify: first message (1 hour old) should be expired
        // Note: batch read stops at first missing record, so we can't read offset 1-2 directly
        // Instead, we check specific offsets individually
        let cf = adapter.db.cf_handle(DB_COLUMN_FAMILY_BROKER).unwrap();

        let key0 = shard_record_key(&namespace, &shard_name, 0);
        let record0 = adapter.db.read::<Record>(cf.clone(), &key0).unwrap();
        assert!(record0.is_none(), "Offset 0 should be expired");

        let key1 = shard_record_key(&namespace, &shard_name, 1);
        let record1 = adapter.db.read::<Record>(cf.clone(), &key1).unwrap();
        assert!(record1.is_some(), "Offset 1 should exist");
        assert_eq!(record1.unwrap().key, Some("key2".to_string()));

        let key2 = shard_record_key(&namespace, &shard_name, 2);
        let record2 = adapter.db.read::<Record>(cf.clone(), &key2).unwrap();
        assert!(record2.is_some(), "Offset 2 should exist");
        assert_eq!(record2.unwrap().key, Some("key3".to_string()));
    }

    /// Test that key index is NOT deleted when the same key has newer messages
    #[tokio::test]
    async fn test_expire_preserves_key_index_for_newer_messages() {
        let adapter = RocksDBStorageAdapter::new(test_storage_driver_rockdb_config());
        let namespace = unique_id();
        let shard_name = "test-shard".to_string();

        adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let now = now_second();

        let messages = vec![
            Record {
                offset: None,
                header: None,
                key: Some("same_key".to_string()),
                data: b"old_data".to_vec().into(),
                tags: Some(vec!["tag1".to_string()]),
                timestamp: now - 3600, // 1 hour ago (will be expired)
                crc_num: 0,
            },
            Record {
                offset: None,
                header: None,
                key: Some("same_key".to_string()), // Same key!
                data: b"new_data".to_vec().into(),
                tags: Some(vec!["tag2".to_string()]),
                timestamp: now - 100, // Recent (will be kept)
                crc_num: 0,
            },
        ];

        adapter
            .batch_write(&namespace, &shard_name, &messages)
            .await
            .unwrap();

        let cf = adapter.db.cf_handle(DB_COLUMN_FAMILY_BROKER).unwrap();
        let key_index_key = key_offset_key(&namespace, &shard_name, "same_key");
        let key_index_bytes = adapter.db.db.get_cf(&cf, &key_index_key).unwrap().unwrap();
        assert_eq!(key_index_bytes.len(), 8);
        let key_index_offset = u64::from_be_bytes(key_index_bytes.as_slice().try_into().unwrap());
        assert_eq!(
            key_index_offset, 1,
            "Key index should point to latest offset 1"
        );

        // Expire old messages (older than 30 minutes)
        let config = MessageExpireConfig::with_timestamp(1800);
        expire_messages_by_timestamp(adapter.db.clone(), &config)
            .await
            .unwrap();

        let record0_key = shard_record_key(&namespace, &shard_name, 0);
        let record0 = adapter.db.read::<Record>(cf.clone(), &record0_key).unwrap();
        assert!(record0.is_none(), "Offset 0 should be expired");

        let record1_key = shard_record_key(&namespace, &shard_name, 1);
        let record1 = adapter.db.read::<Record>(cf.clone(), &record1_key).unwrap();
        assert!(record1.is_some(), "Offset 1 should exist");
        assert_eq!(record1.unwrap().data.as_ref(), b"new_data");

        // CRITICAL: Verify key index still exists and points to offset 1
        let key_index_bytes_after = adapter.db.db.get_cf(&cf, &key_index_key).unwrap().unwrap();
        let key_index_offset_after =
            u64::from_be_bytes(key_index_bytes_after.as_slice().try_into().unwrap());
        assert_eq!(
            key_index_offset_after, 1,
            "Key index should still point to offset 1 after expiration"
        );

        let read_config = metadata_struct::adapter::read_config::ReadConfig {
            max_record_num: 10,
            max_size: u64::MAX,
        };
        let key_records = adapter
            .read_by_key(&namespace, &shard_name, 0, "same_key", &read_config)
            .await
            .unwrap();
        assert_eq!(key_records.len(), 1);
        assert_eq!(key_records[0].data.as_ref(), b"new_data");
    }

    /// Test batch commit logic with large number of deletions
    #[tokio::test]
    async fn test_expire_batch_commit_logic() {
        let adapter = RocksDBStorageAdapter::new(test_storage_driver_rockdb_config());
        let namespace = unique_id();
        let shard_name = "test-shard".to_string();

        adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let now = now_second();

        let messages: Vec<Record> = (0..2500)
            .map(|i| Record {
                offset: None,
                header: None,
                key: Some(format!("key_{}", i)),
                data: format!("data_{}", i).into_bytes().into(),
                tags: Some(vec![format!("tag_{}", i % 10)]),
                timestamp: now - 7200, // 2 hours ago (will be expired)
                crc_num: 0,
            })
            .collect();

        adapter
            .batch_write(&namespace, &shard_name, &messages)
            .await
            .unwrap();

        let cf = adapter.db.cf_handle(DB_COLUMN_FAMILY_BROKER).unwrap();
        let record0_key = shard_record_key(&namespace, &shard_name, 0);
        let record0 = adapter.db.read::<Record>(cf.clone(), &record0_key).unwrap();
        assert!(record0.is_some());

        // Expire messages older than 1 hour
        let config = MessageExpireConfig::with_timestamp(3600);
        expire_messages_by_timestamp(adapter.db.clone(), &config)
            .await
            .unwrap();

        for i in 0..2500 {
            let record_key = shard_record_key(&namespace, &shard_name, i);
            let record = adapter.db.read::<Record>(cf.clone(), &record_key).unwrap();
            assert!(record.is_none(), "Record at offset {} should be expired", i);
        }
    }
}
