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

pub mod delete;
pub mod engine;
pub mod expire;
pub mod index;
pub mod read;
pub mod replica;
pub mod write;

#[cfg(test)]
mod tests {
    use crate::commitlog::memory::engine::MemoryStorageEngine;
    use crate::core::offset::ShardOffsetState;
    use crate::core::test_tool::test_build_memory_engine;
    use bytes::Bytes;
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;
    use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};

    fn cfg() -> AdapterReadConfig {
        AdapterReadConfig {
            max_record_num: 100,
            max_size: 1 << 30,
        }
    }

    fn setup_offsets(engine: &MemoryStorageEngine, shard: &str) {
        engine
            .cache_manager
            .save_offset_state(shard.to_string(), ShardOffsetState::default());
        engine
            .commit_log_offset
            .save_earliest_offset(shard, 0)
            .unwrap();
        engine
            .commit_log_offset
            .save_latest_offset(shard, 0)
            .unwrap();
        engine
            .commit_log_offset
            .save_high_watermark_offset(shard, 0)
            .unwrap();
    }

    fn rec(i: u64) -> AdapterWriteRecord {
        AdapterWriteRecord::new("", Bytes::default())
            .with_key(format!("key{i}"))
            .with_tags(vec!["grp".to_string(), format!("t{i}")])
    }

    #[tokio::test]
    async fn memory_full_lifecycle() {
        let engine = test_build_memory_engine();
        let shard = unique_id();
        setup_offsets(&engine, &shard);
        let config = EngineShardConfig {
            retention_sec: 100,
            max_record_num: None,
            ..Default::default()
        };
        engine.cache_manager.set_shard(EngineShard::new(
            shard.clone(),
            shard.clone(),
            config,
            String::new(),
        ));

        let messages: Vec<AdapterWriteRecord> = (0..10).map(rec).collect();
        engine.batch_write(&shard, &messages).await.unwrap();

        assert_eq!(
            engine
                .read_by_offset(&shard, 0, &cfg())
                .await
                .unwrap()
                .len(),
            10
        );
        assert_eq!(
            engine
                .read_by_tag(&shard, "grp", None, &cfg())
                .await
                .unwrap()
                .len(),
            10
        );
        let r = engine.read_by_key(&shard, b"key5").await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].metadata.offset, 5);

        engine.delete_by_key(&shard, b"key3").await.unwrap();
        engine.delete_by_offset(&shard, 7).await.unwrap();

        assert_eq!(
            engine
                .read_by_offset(&shard, 0, &cfg())
                .await
                .unwrap()
                .len(),
            8
        );
        assert_eq!(
            engine
                .read_by_tag(&shard, "grp", None, &cfg())
                .await
                .unwrap()
                .len(),
            8
        );
        assert!(engine
            .read_by_key(&shard, b"key3")
            .await
            .unwrap()
            .is_empty());
        assert_eq!(engine.read_by_key(&shard, b"key5").await.unwrap().len(), 1);
        assert!(engine
            .read_by_tag(&shard, "t7", None, &cfg())
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_tag(&shard, "t5", None, &cfg())
                .await
                .unwrap()
                .len(),
            1
        );

        let more: Vec<AdapterWriteRecord> = (10..15).map(rec).collect();
        engine.batch_write(&shard, &more).await.unwrap();
        assert_eq!(
            engine
                .read_by_offset(&shard, 0, &cfg())
                .await
                .unwrap()
                .len(),
            13
        );

        let old_ts = now_second() - 110;
        let shard_ref = engine.shards.get(&shard).unwrap();
        for offset in 0..3 {
            shard_ref.data.get_mut(&offset).unwrap().metadata.create_t = old_ts;
        }
        drop(shard_ref);
        engine.scan_and_delete_expire_data();

        assert_eq!(
            engine
                .commit_log_offset
                .get_earliest_offset(&shard)
                .unwrap(),
            3
        );
        let after = engine.read_by_offset(&shard, 0, &cfg()).await.unwrap();
        assert_eq!(after.len(), 10);
        assert_eq!(after[0].metadata.offset, 4);
        assert_eq!(
            engine
                .read_by_tag(&shard, "grp", None, &cfg())
                .await
                .unwrap()
                .len(),
            10
        );
        assert!(engine
            .read_by_key(&shard, b"key0")
            .await
            .unwrap()
            .is_empty());
        assert!(engine
            .read_by_key(&shard, b"key2")
            .await
            .unwrap()
            .is_empty());
        assert_eq!(engine.read_by_key(&shard, b"key5").await.unwrap().len(), 1);
        assert_eq!(engine.read_by_key(&shard, b"key10").await.unwrap().len(), 1);
        assert!(engine
            .read_by_tag(&shard, "t0", None, &cfg())
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_tag(&shard, "t5", None, &cfg())
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn memory_key_compaction() {
        let engine = test_build_memory_engine();
        let shard = unique_id();
        setup_offsets(&engine, &shard);

        let r1 = AdapterWriteRecord::new("", Bytes::default())
            .with_key("k")
            .with_tags(vec!["x".to_string()]);
        let r2 = AdapterWriteRecord::new("", Bytes::default())
            .with_key("k")
            .with_tags(vec!["y".to_string()]);
        engine.write(&shard, &r1).await.unwrap();
        engine.write(&shard, &r2).await.unwrap();

        let r = engine.read_by_key(&shard, b"k").await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].metadata.offset, 1);
        assert_eq!(
            engine
                .read_by_offset(&shard, 0, &cfg())
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(engine
            .read_by_tag(&shard, "x", None, &cfg())
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_tag(&shard, "y", None, &cfg())
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn memory_evict_by_size() {
        let engine = test_build_memory_engine();
        let shard = unique_id();
        setup_offsets(&engine, &shard);

        let messages: Vec<AdapterWriteRecord> = (0..20).map(rec).collect();
        engine.batch_write(&shard, &messages).await.unwrap();

        let shard_ref = engine.shards.get(&shard).unwrap().clone();
        let discard = (20.0 * engine.config.evict_ratio) as u64;
        engine.evict_by_size(&shard, 10, &shard_ref).unwrap();

        assert_eq!(shard_ref.data.len() as u64, 20 - discard);
        assert_eq!(
            engine
                .commit_log_offset
                .get_earliest_offset(&shard)
                .unwrap(),
            discard
        );
        for offset in 0..discard {
            assert!(!shard_ref.data.contains_key(&offset));
        }
        assert!(shard_ref.data.contains_key(&discard));
    }

    #[tokio::test]
    async fn memory_get_offset_by_timestamp() {
        let engine = test_build_memory_engine();
        let shard = unique_id();
        setup_offsets(&engine, &shard);

        let messages: Vec<AdapterWriteRecord> = (0..5).map(rec).collect();
        engine.batch_write(&shard, &messages).await.unwrap();

        let shard_ref = engine.shards.get(&shard).unwrap();
        for (offset, ts) in [(0u64, 100u64), (1, 200), (2, 300), (3, 400), (4, 500)] {
            shard_ref.data.get_mut(&offset).unwrap().metadata.create_t = ts;
        }
        drop(shard_ref);

        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 250, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 100, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 50, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 600, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 600, AdapterOffsetStrategy::Latest)
                .await
                .unwrap(),
            5
        );
    }

    #[tokio::test]
    async fn memory_drop_shard() {
        let engine = test_build_memory_engine();
        let shard_a = unique_id();
        let shard_b = unique_id();
        setup_offsets(&engine, &shard_a);
        setup_offsets(&engine, &shard_b);

        let messages: Vec<AdapterWriteRecord> = (0..5).map(rec).collect();
        engine.batch_write(&shard_a, &messages).await.unwrap();
        engine.batch_write(&shard_b, &messages).await.unwrap();
        assert_eq!(
            engine
                .read_by_offset(&shard_a, 0, &cfg())
                .await
                .unwrap()
                .len(),
            5
        );

        engine.remove_indexes(&shard_a);
        assert!(engine
            .read_by_offset(&shard_a, 0, &cfg())
            .await
            .unwrap()
            .is_empty());
        assert!(engine
            .read_by_key(&shard_a, b"key0")
            .await
            .unwrap()
            .is_empty());
        assert!(engine
            .read_by_tag(&shard_a, "grp", None, &cfg())
            .await
            .unwrap()
            .is_empty());

        engine.delete_by_shard(&shard_b);
        assert!(engine
            .read_by_offset(&shard_b, 0, &cfg())
            .await
            .unwrap()
            .is_empty());

        engine.delete_by_segment(&shard_a, 0);
    }

    #[tokio::test]
    async fn memory_replica_log_flow() {
        use crate::core::error::StorageEngineError;
        use crate::isr::log::ReplicaLog;
        use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};

        let engine = test_build_memory_engine();
        let shard = unique_id();
        setup_offsets(&engine, &shard);

        let make = |offset: u64| StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(format!("d{offset}")),
        };

        engine
            .append_at(&shard, 0, 0, vec![make(0), make(1), make(2), make(3)])
            .await
            .unwrap();
        assert_eq!(engine.latest_offset(&shard, 0).unwrap(), 4);
        assert_eq!(engine.log_start_offset(&shard, 0).unwrap(), 0);

        let err = engine.append_at(&shard, 0, 9, vec![make(9)]).await;
        assert!(matches!(err, Err(StorageEngineError::OutOfOrder(..))));

        let read = engine.read_from(&shard, 0, 1, 1 << 20).await.unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].metadata.offset, 1);

        engine.update_high_watermark(&shard, 3).unwrap();
        assert_eq!(
            engine
                .commit_log_offset
                .get_high_watermark_offset(&shard)
                .unwrap(),
            3
        );

        engine.truncate_to(&shard, 0, 1).await.unwrap();
        assert_eq!(engine.latest_offset(&shard, 0).unwrap(), 2);
        assert_eq!(
            engine.read_from(&shard, 0, 0, 1 << 20).await.unwrap().len(),
            2
        );

        engine.clear(&shard, 0).await.unwrap();
        assert_eq!(engine.latest_offset(&shard, 0).unwrap(), 0);
        assert!(engine
            .read_from(&shard, 0, 0, 1 << 20)
            .await
            .unwrap()
            .is_empty());
    }
}
