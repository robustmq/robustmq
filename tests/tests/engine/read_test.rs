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
    use crate::mqtt::protocol::common::create_test_env;
    use admin_server::engine::shard::ShardCreateReq;
    use broker_core::cache::BrokerCacheManager;
    use bytes::Bytes;
    use common_base::tools::{now_second, unique_id};
    use common_base::utils::serialize::{self, deserialize};
    use common_config::config::BrokerConfig;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::storage::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::storage_record::StorageRecord;
    use protocol::storage::codec::StorageEnginePacket;
    use protocol::storage::protocol::{
        ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType, WriteReq,
        WriteReqBody,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::core::cache::StorageCacheManager;
    use tokio::time::sleep;

    #[tokio::test]
    async fn shard_test_by_segment() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineSegment"}"#.to_string();
        shard_test(config).await
    }

    #[tokio::test]
    async fn shard_test_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#.to_string();
        shard_test(config).await
    }

    #[tokio::test]
    async fn shard_test_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#.to_string();
        shard_test(config).await
    }

    async fn shard_test(config: String) {
        let client = create_test_env().await;
        let shard_name = unique_id();

        let create_result = client
            .create_shard(&ShardCreateReq {
                shard_name: shard_name.clone(),
                config,
            })
            .await
            .unwrap();
        println!("{:?}", create_result);
        sleep(Duration::from_secs(10)).await;

        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let node_id = 1;
        broker_cache.add_node(BrokerNode {
            node_id,
            engine_addr: "127.0.0.1:1778".to_string(),
            ..Default::default()
        });
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let client = Arc::new(ClientConnectionManager::new(cache_manager.clone(), 2));

        let mut messages = Vec::new();
        for i in 0..10 {
            let record = AdapterWriteRecord {
                pkid: 100 + i,
                key: Some(format!("key-{}", i)),
                tags: Some(vec![format!("tag-{}", i % 3)]),
                data: Bytes::from(format!("data-{}", i)),
                timestamp: now_second(),
                ..Default::default()
            };
            messages.push(serialize::serialize(&record).unwrap());
        }

        let write_req = WriteReq::new(WriteReqBody::new(shard_name.clone(), messages));
        let write_packet = StorageEnginePacket::WriteReq(write_req);
        let write_resp = client.write_send(node_id, write_packet).await.unwrap();

        match write_resp {
            StorageEnginePacket::WriteResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "WriteResp error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert_eq!(resp.body.status.len(), 1);
                assert_eq!(resp.body.status[0].shard_name, shard_name);
                assert_eq!(resp.body.status[0].messages.len(), 10);
            }
            _ => panic!("Expected WriteResp"),
        }

        let read_req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.clone(),
            ReadType::Offset,
            false,
            ReadReqFilter::by_offset(0),
            ReadReqOptions::new(1024 * 1024, 5),
        )]));
        let read_packet = StorageEnginePacket::ReadReq(read_req);
        let read_resp = client.read_send(node_id, read_packet).await.unwrap();
        match read_resp {
            StorageEnginePacket::ReadResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "ReadResp(Offset) error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert_eq!(resp.body.messages.len(), 5);
                for (idx, record_bytes) in resp.body.messages.iter().enumerate() {
                    let record: StorageRecord = deserialize(record_bytes).unwrap();
                    assert_eq!(record.metadata.offset, idx as u64);
                }
            }
            _ => panic!("Expected ReadResp"),
        }

        let read_req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.clone(),
            ReadType::Key,
            false,
            ReadReqFilter {
                offset: Some(0),
                key: Some("key-3".to_string()),
                ..Default::default()
            },
            ReadReqOptions::new(1024 * 1024, 10),
        )]));
        let read_packet = StorageEnginePacket::ReadReq(read_req);
        let read_resp = client.read_send(node_id, read_packet).await.unwrap();
        match read_resp {
            StorageEnginePacket::ReadResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "ReadResp(Key) error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert_eq!(resp.body.messages.len(), 1);
                let record: StorageRecord = deserialize(&resp.body.messages[0]).unwrap();
                assert_eq!(record.metadata.key.unwrap(), "key-3");
            }
            _ => panic!("Expected ReadResp"),
        }

        let read_req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.clone(),
            ReadType::Tag,
            false,
            ReadReqFilter {
                offset: Some(0),
                tag: Some("tag-1".to_string()),
                ..Default::default()
            },
            ReadReqOptions::new(1024 * 1024, 10),
        )]));
        let read_packet = StorageEnginePacket::ReadReq(read_req);
        let read_resp = client.read_send(node_id, read_packet).await.unwrap();
        match read_resp {
            StorageEnginePacket::ReadResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "ReadResp(Tag) error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert!(resp.body.messages.len() >= 3);
                for record_bytes in resp.body.messages.iter() {
                    let record: StorageRecord = deserialize(record_bytes).unwrap();
                    assert!(record.metadata.tags.unwrap().contains(&"tag-1".to_string()));
                }
            }
            _ => panic!("Expected ReadResp"),
        }
    }
}
