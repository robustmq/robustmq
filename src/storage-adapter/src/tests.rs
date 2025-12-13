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

use std::collections::HashMap;

use metadata_struct::adapter::{read_config::ReadConfig, record::Record};

use crate::storage::{ArcStorageAdapter, ShardInfo};

pub async fn test_shard_lifecycle(adapter: ArcStorageAdapter) {
    let shard1 = ShardInfo {
        shard_name: "shard1".to_string(),
        replica_num: 3,
        ..Default::default()
    };
    adapter.create_shard(&shard1).await.unwrap();
    adapter
        .create_shard(&ShardInfo {
            shard_name: "shard2".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(adapter.list_shard("").await.unwrap().len(), 2);
    assert_eq!(
        adapter.list_shard("shard1").await.unwrap()[0].replica_num,
        3
    );

    adapter.delete_shard("shard1").await.unwrap();
    assert_eq!(adapter.list_shard("").await.unwrap().len(), 1);
    assert_eq!(adapter.list_shard("shard1").await.unwrap().len(), 0);
}

pub async fn test_write_and_read(adapter: ArcStorageAdapter) {
    let cfg = ReadConfig {
        max_record_num: 10,
        max_size: 1024 * 1024,
    };

    adapter
        .create_shard(&ShardInfo {
            shard_name: "s".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut r1 = Record::from_bytes(b"msg1".to_vec());
    r1.key = Some("k1".to_string());
    r1.tags = Some(vec!["a".to_string(), "c".to_string()]);
    r1.timestamp = 1000;

    let mut r2 = Record::from_bytes(b"msg2".to_vec());
    r2.key = Some("k2".to_string());
    r2.tags = Some(vec!["b".to_string(), "c".to_string()]);
    r2.timestamp = 2000;

    assert_eq!(
        adapter.batch_write("s", &[r1, r2]).await.unwrap(),
        vec![0, 1]
    );
    assert_eq!(adapter.read_by_offset("s", 0, &cfg).await.unwrap().len(), 2);
    assert_eq!(adapter.read_by_offset("s", 1, &cfg).await.unwrap().len(), 1);

    assert_eq!(
        adapter
            .read_by_tag("s", "a", None, &cfg)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        adapter
            .read_by_tag("s", "c", None, &cfg)
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        adapter
            .read_by_tag("s", "b", Some(1), &cfg)
            .await
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        adapter.read_by_key("s", "k2").await.unwrap()[0].data,
        b"msg2".to_vec()
    );
    assert_eq!(adapter.read_by_key("s", "k3").await.unwrap().len(), 0);

    assert_eq!(
        adapter
            .get_offset_by_timestamp("s", 1500)
            .await
            .unwrap()
            .unwrap()
            .offset,
        1
    );
    assert!(adapter
        .get_offset_by_timestamp("s", 5000)
        .await
        .unwrap()
        .is_none());
}

pub async fn test_consumer_group_offset(adapter: ArcStorageAdapter) {
    adapter
        .create_shard(&ShardInfo {
            shard_name: "s1".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    adapter
        .create_shard(&ShardInfo {
            shard_name: "s2".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    adapter
        .commit_offset(
            "g1",
            &HashMap::from([("s1".into(), 100), ("s2".into(), 200)]),
        )
        .await
        .unwrap();

    let offsets = adapter.get_offset_by_group("g1").await.unwrap();
    assert_eq!(offsets.len(), 2);
    assert_eq!(
        offsets
            .iter()
            .find(|o| o.shard_name == "s1")
            .unwrap()
            .offset,
        100
    );
    assert_eq!(
        offsets
            .iter()
            .find(|o| o.shard_name == "s2")
            .unwrap()
            .offset,
        200
    );

    adapter
        .commit_offset("g1", &HashMap::from([("s1".into(), 150)]))
        .await
        .unwrap();
    let offsets = adapter.get_offset_by_group("g1").await.unwrap();
    assert_eq!(
        offsets
            .iter()
            .find(|o| o.shard_name == "s1")
            .unwrap()
            .offset,
        150
    );

    adapter
        .commit_offset("g2", &HashMap::from([("s1".into(), 300)]))
        .await
        .unwrap();
    assert_eq!(adapter.get_offset_by_group("g2").await.unwrap().len(), 1);

    assert_eq!(adapter.get_offset_by_group("g3").await.unwrap().len(), 0);

    assert!(adapter
        .commit_offset("g1", &HashMap::from([("s3".into(), 100)]))
        .await
        .is_ok());
}

pub async fn test_timestamp_index_with_multiple_entries(adapter: ArcStorageAdapter) {
    let cfg = ReadConfig {
        max_record_num: 100,
        max_size: 1024 * 1024,
    };

    adapter
        .create_shard(&ShardInfo {
            shard_name: "s".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut records = Vec::new();
    for i in 0..15000 {
        let mut r = Record::from_bytes(format!("msg{}", i).into_bytes());
        r.timestamp = 1000 + i;
        records.push(r);
    }

    let offsets = adapter.batch_write("s", &records).await.unwrap();
    assert_eq!(offsets.len(), 15000);
    assert_eq!(offsets[0], 0);
    assert_eq!(offsets[14999], 14999);

    let result = adapter.get_offset_by_timestamp("s", 1000).await.unwrap();
    assert_eq!(result.unwrap().offset, 0);

    let result = adapter.get_offset_by_timestamp("s", 3500).await.unwrap();
    assert_eq!(result.unwrap().offset, 2500);

    let result = adapter.get_offset_by_timestamp("s", 6000).await.unwrap();
    assert_eq!(result.unwrap().offset, 5000);

    let result = adapter.get_offset_by_timestamp("s", 8000).await.unwrap();
    assert_eq!(result.unwrap().offset, 7000);

    let result = adapter.get_offset_by_timestamp("s", 11000).await.unwrap();
    assert_eq!(result.unwrap().offset, 10000);

    let result = adapter.get_offset_by_timestamp("s", 14500).await.unwrap();
    assert_eq!(result.unwrap().offset, 13500);

    let result = adapter.get_offset_by_timestamp("s", 500).await.unwrap();
    assert_eq!(result.unwrap().offset, 0);

    let result = adapter.get_offset_by_timestamp("s", 20000).await.unwrap();
    assert!(result.is_none());

    let read_result = adapter.read_by_offset("s", 5000, &cfg).await.unwrap();
    assert!(!read_result.is_empty());
    assert_eq!(read_result[0].timestamp, 6000);
}
