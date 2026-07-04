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
pub mod read;
pub mod replica;
pub mod write;

#[cfg(test)]
mod tests {
    use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
    use crate::core::test_tool::test_build_rocksdb_engine;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;

    fn cfg() -> AdapterReadConfig {
        AdapterReadConfig {
            max_record_num: 100,
            max_size: 1 << 30,
        }
    }

    fn setup(engine: &RocksDBStorageEngine, shard: &str) {
        engine
            .commitlog_offset
            .save_earliest_offset(shard, 0)
            .unwrap();
        engine
            .commitlog_offset
            .save_latest_offset(shard, 0)
            .unwrap();
    }

    fn rec(i: u64) -> AdapterWriteRecord {
        AdapterWriteRecord::new("", Bytes::default())
            .with_key(format!("key{i}"))
            .with_tags(vec!["grp".to_string(), format!("t{i}")])
    }

    #[tokio::test]
    async fn rocksdb_full_lifecycle() {
        let engine = test_build_rocksdb_engine();
        let shard = unique_id();
        setup(&engine, &shard);

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
        assert_eq!(engine.read_by_key(&shard, b"key12").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rocksdb_key_compaction() {
        let engine = test_build_rocksdb_engine();
        let shard = unique_id();
        setup(&engine, &shard);

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
    async fn rocksdb_batch_delete() {
        let engine = test_build_rocksdb_engine();
        let shard = unique_id();
        setup(&engine, &shard);

        let messages: Vec<AdapterWriteRecord> = (0..10).map(rec).collect();
        engine.batch_write(&shard, &messages).await.unwrap();

        engine
            .delete_by_keys(&shard, &[b"key1".as_ref(), b"key2".as_ref()])
            .await
            .unwrap();
        engine.delete_by_offsets(&shard, &[5, 6]).await.unwrap();

        assert_eq!(
            engine
                .read_by_offset(&shard, 0, &cfg())
                .await
                .unwrap()
                .len(),
            6
        );
        assert!(engine
            .read_by_key(&shard, b"key1")
            .await
            .unwrap()
            .is_empty());
        assert_eq!(engine.read_by_key(&shard, b"key0").await.unwrap().len(), 1);
        assert!(engine
            .read_by_tag(&shard, "t5", None, &cfg())
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            engine
                .read_by_tag(&shard, "t0", None, &cfg())
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn rocksdb_get_offset_by_timestamp() {
        let engine = test_build_rocksdb_engine();
        let shard = unique_id();
        setup(&engine, &shard);

        let messages: Vec<AdapterWriteRecord> = (0..10).map(rec).collect();
        engine.batch_write(&shard, &messages).await.unwrap();

        // A timestamp every record satisfies resolves to the earliest matching offset.
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, 1, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            0
        );

        // A far-future timestamp matches nothing, so the strategy fallback decides.
        let future = 9_999_999_999u64;
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, future, AdapterOffsetStrategy::Earliest)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            engine
                .get_offset_by_timestamp(&shard, future, AdapterOffsetStrategy::Latest)
                .await
                .unwrap(),
            10
        );
    }

    #[tokio::test]
    async fn rocksdb_delete_shard_and_segment() {
        let engine = test_build_rocksdb_engine();
        let shard_a = unique_id();
        let shard_b = unique_id();
        setup(&engine, &shard_a);
        setup(&engine, &shard_b);

        let messages: Vec<AdapterWriteRecord> = (0..5).map(rec).collect();
        engine.batch_write(&shard_a, &messages).await.unwrap();
        engine.batch_write(&shard_b, &messages).await.unwrap();

        engine.delete_by_segment(&shard_a, 0).unwrap();
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

        engine.delete_by_shard(&shard_b).unwrap();
        // delete_by_shard wipes meta too; key/tag indices are gone outright.
        assert!(engine
            .read_by_key(&shard_b, b"key0")
            .await
            .unwrap()
            .is_empty());
        assert!(engine
            .read_by_tag(&shard_b, "grp", None, &cfg())
            .await
            .unwrap()
            .is_empty());
        // Restore the latest marker to prove the records themselves are gone.
        engine
            .commitlog_offset
            .save_latest_offset(&shard_b, 5)
            .unwrap();
        assert!(engine
            .read_by_offset(&shard_b, 0, &cfg())
            .await
            .unwrap()
            .is_empty());
    }
}
