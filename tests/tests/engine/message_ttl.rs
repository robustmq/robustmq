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
    use broker_core::cache::NodeCacheManager;
    use bytes::Bytes;
    use common_base::tools::now_second;
    use common_base::utils::serialize::{self, deserialize};
    use common_base::uuid::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::storage::record::StorageRecord;
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
    async fn message_ttl_test_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#.to_string();
        message_ttl_test(config).await;
    }

    #[tokio::test]
    async fn message_ttl_test_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#.to_string();
        message_ttl_test(config).await;
    }

    async fn message_ttl_test(config: String) {
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

        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let node_id = 1;
        broker_cache.add_node(BrokerNode {
            node_id,
            engine_addr: "127.0.0.1:1779".to_string(),
            ..Default::default()
        });
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let client = Arc::new(ClientConnectionManager::new(cache_manager.clone(), 2));

        let now = now_second();

        // offset 0: not expired (expires 5 minutes from now)
        let alive = AdapterWriteRecord::new("", Bytes::from("alive")).with_expire_at(now + 300);

        // offset 1: already expired (expired 1 second ago)
        let expired = AdapterWriteRecord::new("", Bytes::from("expired")).with_expire_at(now - 1);

        let messages = vec![
            serialize::serialize(&alive).unwrap(),
            serialize::serialize(&expired).unwrap(),
        ];

        let write_req = WriteReq::new(WriteReqBody::new(shard_name.clone(), messages));
        let write_resp = client
            .write_send(node_id, StorageEnginePacket::WriteReq(write_req))
            .await
            .unwrap();

        match write_resp {
            StorageEnginePacket::WriteResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "WriteResp error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert_eq!(resp.body.status[0].messages.len(), 2);
            }
            _ => panic!("Expected WriteResp"),
        }

        // Read from offset 0, limit 10 — expect only the non-expired message
        let read_req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
            shard_name.clone(),
            ReadType::Offset,
            false,
            ReadReqFilter::by_offset(0),
            ReadReqOptions::new(1024 * 1024, 10),
        )]));
        let read_resp = client
            .read_send(node_id, StorageEnginePacket::ReadReq(read_req))
            .await
            .unwrap();

        match read_resp {
            StorageEnginePacket::ReadResp(resp) => {
                if let Some(error) = resp.header.error {
                    panic!(
                        "ReadResp error: code={}, message={}",
                        error.code, error.error
                    );
                }
                assert_eq!(
                    resp.body.messages.len(),
                    1,
                    "expected 1 non-expired message, got {}",
                    resp.body.messages.len()
                );
                let record: StorageRecord = deserialize(&resp.body.messages[0]).unwrap();
                assert_eq!(record.data, Bytes::from("alive"));
                assert_eq!(record.metadata.offset, 0);
            }
            _ => panic!("Expected ReadResp"),
        }
    }
}
