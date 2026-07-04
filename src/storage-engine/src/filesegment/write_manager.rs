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

use super::write_io_work::create_io_thread;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::filesegment::SegmentIdentity;
use bytes::Bytes;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_read_config::AdapterWriteRespRow;
use metadata_struct::storage::record::{StorageHeader, StorageRecordProtocolData};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;
use twox_hash::XxHash32;

pub struct WriteChannelData {
    pub segment_iden: SegmentIdentity,
    pub data_list: Vec<WriteChannelDataRecord>,
    pub resp_sx: oneshot::Sender<SegmentWriteResp>,
}

#[derive(Debug, Clone)]
pub struct WriteChannelDataRecord {
    pub pkid: u64,
    pub header: Option<Vec<StorageHeader>>,
    pub key: Option<Bytes>,
    pub value: Bytes,
    pub tags: Option<Vec<String>>,
    pub expire_at: u64,
    pub protocol_data: Option<StorageRecordProtocolData>,
}

#[derive(Default, Debug, Clone)]
pub struct SegmentWriteResp {
    pub offsets: Vec<AdapterWriteRespRow>,
    pub last_offset: u64,
    pub error: Option<String>,
}

pub struct WriteManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
    io_num: u32,
    io_thread: DashMap<u32, Sender<WriteChannelData>>,
}

impl WriteManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        client_pool: Arc<ClientPool>,
        io_num: u32,
    ) -> Self {
        WriteManager {
            rocksdb_engine_handler,
            cache_manager,
            client_pool,
            io_num,
            io_thread: DashMap::with_capacity(2),
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        for i in 0..self.io_num {
            let (data_sender, data_recv) = mpsc::channel::<WriteChannelData>(1000);
            create_io_thread(
                self.rocksdb_engine_handler.clone(),
                self.cache_manager.clone(),
                self.client_pool.clone(),
                data_recv,
                stop_send.clone(),
                i,
            );
            self.io_thread.insert(i, data_sender);
        }
    }

    pub async fn write(
        &self,
        segment_iden: &SegmentIdentity,
        data_list: Vec<WriteChannelDataRecord>,
    ) -> Result<SegmentWriteResp, StorageEngineError> {
        if self.io_thread.is_empty() {
            return Err(StorageEngineError::NoAvailableIoThread);
        }

        let work_num = self.hash_string(&segment_iden.shard_name) % self.io_num;
        let Some(sender) = self.io_thread.get(&work_num) else {
            return Err(StorageEngineError::NoAvailableIoThread);
        };

        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();
        sender
            .send(WriteChannelData {
                segment_iden: segment_iden.clone(),
                data_list,
                resp_sx: sx,
            })
            .await?;

        let time_res: Result<SegmentWriteResp, oneshot::error::RecvError> =
            timeout(Duration::from_secs(30), rx).await?;
        Ok(time_res?)
    }

    fn hash_string(&self, shard: &str) -> u32 {
        let mut hasher = XxHash32::default();
        hasher.write(shard.as_bytes());
        hasher.finish() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_init_segment;
    use common_config::storage::StorageType;

    #[tokio::test]
    async fn write_manager_write_test() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let client_pool = Arc::new(ClientPool::new(100));
        let write_manager =
            WriteManager::new(rocksdb.clone(), cache_manager.clone(), client_pool, 3);

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data_list = (0..5u64)
            .map(|i| WriteChannelDataRecord {
                pkid: i,
                header: None,
                key: None,
                tags: None,
                value: Bytes::from(format!("data-{}", i)),
                protocol_data: None,
                expire_at: 0,
            })
            .collect();

        let resp = write_manager.write(&segment_iden, data_list).await.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.offsets.len(), 5);
        assert_eq!(resp.last_offset, 4);
    }

    #[tokio::test]
    async fn write_manager_no_io_thread_test() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let client_pool = Arc::new(ClientPool::new(100));

        // start() not called — io_thread is empty
        let write_manager = WriteManager::new(rocksdb, cache_manager, client_pool, 3);

        let result = write_manager
            .write(
                &segment_iden,
                vec![WriteChannelDataRecord {
                    pkid: 1,
                    header: None,
                    key: None,
                    tags: None,
                    value: Bytes::from("v"),
                    protocol_data: None,
                    expire_at: 0,
                }],
            )
            .await;

        assert!(matches!(
            result.unwrap_err(),
            StorageEngineError::NoAvailableIoThread
        ));
    }
}
