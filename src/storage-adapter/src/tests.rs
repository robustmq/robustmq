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

use crate::driver::ArcStorageAdapter;
use common_base::uuid::unique_id;
use metadata_struct::adapter::adapter_offset::{AdapterOffsetStrategy, AdapterShardInfo};
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::shard::EngineShardConfig;

pub async fn test_shard_lifecycle(adapter: ArcStorageAdapter) {
    let shard1_name = unique_id();
    let shard2_name = unique_id();

    let shard1 = AdapterShardInfo {
        shard_name: shard1_name.clone(),
        config: EngineShardConfig::default(),
        ..Default::default()
    };
    adapter.create_shard(&shard1).await.unwrap();
    adapter
        .create_shard(&AdapterShardInfo {
            shard_name: shard2_name.clone(),
            config: EngineShardConfig::default(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(adapter.list_shard(None).await.unwrap().len(), 2);
    assert_eq!(
        adapter.list_shard(Some(shard1_name.clone())).await.unwrap()[0]
            .config
            .replica_num,
        1
    );

    adapter.delete_shard(&shard1_name).await.unwrap();
    assert_eq!(adapter.list_shard(None).await.unwrap().len(), 1);
    assert_eq!(
        adapter.list_shard(Some(shard1_name)).await.unwrap().len(),
        0
    );
}

pub async fn test_write_and_read(adapter: ArcStorageAdapter) {
    let shard_name = unique_id();
    let cfg = AdapterReadConfig {
        max_record_num: 10,
        max_size: 1024 * 1024,
    };

    adapter
        .create_shard(&AdapterShardInfo {
            shard_name: shard_name.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let r1 = AdapterWriteRecord::new(&shard_name, b"msg1".as_ref())
        .with_key("k1")
        .with_tags(vec!["a".to_string(), "c".to_string()]);

    let r2 = AdapterWriteRecord::new(&shard_name, b"msg2".as_ref())
        .with_key("k2")
        .with_tags(vec!["b".to_string(), "c".to_string()]);

    let offsets: Vec<u64> = adapter
        .batch_write(&shard_name, &[r1, r2])
        .await
        .unwrap()
        .iter()
        .map(|raw| raw.offset)
        .collect();
    assert_eq!(offsets, vec![0, 1]);
    assert_eq!(
        adapter
            .read_by_offset(&shard_name, 0, &cfg)
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        adapter
            .read_by_offset(&shard_name, 1, &cfg)
            .await
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        adapter
            .read_by_tag(&shard_name, "a", None, &cfg)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        adapter
            .read_by_tag(&shard_name, "c", None, &cfg)
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        adapter
            .read_by_tag(&shard_name, "b", Some(1), &cfg)
            .await
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        adapter.read_by_key(&shard_name, "k2").await.unwrap()[0].data,
        b"msg2".to_vec()
    );
    assert_eq!(
        adapter.read_by_key(&shard_name, "k3").await.unwrap().len(),
        0
    );
}

pub async fn test_timestamp_index_with_multiple_entries(adapter: ArcStorageAdapter) {
    let shard_name = unique_id();
    let cfg = AdapterReadConfig {
        max_record_num: 100,
        max_size: 1024 * 1024,
    };

    adapter
        .create_shard(&AdapterShardInfo {
            shard_name: shard_name.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut records = Vec::new();
    for i in 0..15000u64 {
        let r = AdapterWriteRecord::new(&shard_name, format!("msg{}", i).into_bytes())
            .with_pkid(1000 + i);
        records.push(r);
    }

    let offsets = adapter.batch_write(&shard_name, &records).await.unwrap();
    assert_eq!(offsets.len(), 15000);
    assert_eq!(offsets[0].offset, 0);
    assert_eq!(offsets[14999].offset, 14999);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 1000, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 0);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 3500, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 2500);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 6000, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 5000);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 8000, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 7000);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 11000, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 10000);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 14500, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 13500);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 500, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();
    assert_eq!(result, 0);

    let result = adapter
        .get_offset_by_timestamp(&shard_name, 20000, AdapterOffsetStrategy::Earliest)
        .await
        .unwrap();

    assert_eq!(result, 0);

    let read_result = adapter
        .read_by_offset(&shard_name, 5000, &cfg)
        .await
        .unwrap();
    assert!(!read_result.is_empty());
    assert_eq!(read_result[0].metadata.create_t, 6000);
}
