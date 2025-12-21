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

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::record::{StorageEngineRecord, StorageEngineRecordMetadata};
use crate::segment::file::{open_segment_write, SegmentFile};
use crate::segment::index::build::try_trigger_build_index;
use crate::segment::manager::SegmentFileManager;
use crate::segment::offset::{get_shard_offset, save_shard_offset};
use crate::segment::SegmentIdentity;
use bytes::Bytes;
use common_base::tools::now_second;
use dashmap::DashMap;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};
use tracing::{error, info};
use twox_hash::XxHash32;

#[derive(Clone)]
pub struct SegmentWrite {
    pub data_sender: Sender<WriteChannelData>,
    pub stop_sender: broadcast::Sender<bool>,
}

/// the data to be sent to the segment write thread
pub struct WriteChannelData {
    pub segment_iden: SegmentIdentity,
    pub data_list: Vec<WriteChannelDataRecord>,
    pub resp_sx: oneshot::Sender<SegmentWriteResp>,
}

pub struct WriteChannelDataRecord {
    pub pkid: u64,
    pub key: Option<String>,
    pub value: Bytes,
    pub tags: Option<Vec<String>>,
}

/// the response of the write request from the segment write thread
#[derive(Default, Debug, Clone)]
pub struct SegmentWriteResp {
    pub offsets: HashMap<u64, u64>,
    pub last_offset: u64,
    pub error: Option<String>,
}

pub struct WriteManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<StorageCacheManager>,
    io_num: u32,
    io_thread: DashMap<u32, Sender<WriteChannelData>>,
}

impl WriteManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        segment_file_manager: Arc<SegmentFileManager>,
        cache_manager: Arc<StorageCacheManager>,
        io_num: u32,
    ) -> Self {
        WriteManager {
            rocksdb_engine_handler,
            segment_file_manager,
            cache_manager,
            io_num,
            io_thread: DashMap::with_capacity(2),
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        for i in 0..self.io_num {
            let (data_sender, data_recv) = mpsc::channel::<WriteChannelData>(1000);
            let io_work = Arc::new(IoWork::new(self.rocksdb_engine_handler.clone(), i));
            create_io_thread(
                io_work,
                self.rocksdb_engine_handler.clone(),
                self.segment_file_manager.clone(),
                self.cache_manager.clone(),
                data_recv,
                stop_send.clone(),
            );
            self.io_thread.insert(i, data_sender);
        }
    }

    pub async fn write(
        &self,
        segment_iden: &SegmentIdentity,
        data_list: Vec<WriteChannelDataRecord>,
    ) -> Result<SegmentWriteResp, StorageEngineError> {
        if self.io_thread.len() == 0 {
            return Err(StorageEngineError::NoAvailableIoThread);
        }

        let work_num = self.hash_string(&segment_iden.shard_name);
        let Some(sender) = self.io_thread.get(&work_num) else {
            return Err(StorageEngineError::NoAvailableIoThread);
        };

        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();
        let data = WriteChannelData {
            segment_iden: segment_iden.clone(),
            data_list,
            resp_sx: sx,
        };
        sender.send(data).await?;

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

#[derive(Clone)]
pub struct IoWork {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    offset_data: DashMap<String, u64>,
}

impl IoWork {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>, io_seq: u32) -> Self {
        info!("io worker {} start success", io_seq);
        IoWork {
            rocksdb_engine_handler,
            offset_data: DashMap::with_capacity(16),
        }
    }

    pub fn get_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(offset) = self.offset_data.get(shard_name) {
            return Ok(*offset);
        }

        let offset = get_shard_offset(&self.rocksdb_engine_handler, shard_name)?;
        self.offset_data.insert(shard_name.to_string(), offset);
        Ok(offset)
    }

    pub fn save_offset(&self, shard_name: &str, offset: u64) {
        self.offset_data.insert(shard_name.to_string(), offset);
    }

    pub fn flush_offset(&self, shard_name: &str) -> Result<(), StorageEngineError> {
        if let Some(offset) = self.offset_data.get(shard_name) {
            save_shard_offset(&self.rocksdb_engine_handler, shard_name, *offset)?;
        }
        Ok(())
    }
}

pub fn create_io_thread(
    io_work: Arc<IoWork>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<StorageCacheManager>,
    mut data_recv: Receiver<WriteChannelData>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        loop {
            let mut stop_recv = stop_send.subscribe();

            // check io thread exist
            match stop_recv.try_recv() {
                Ok(bl) => {
                    if bl {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    break;
                }
                Err(_) => {}
            }

            // batch recv data
            let mut results = Vec::new();
            loop {
                match data_recv.try_recv() {
                    Ok(data) => {
                        results.push(data);
                        if results.len() >= 100 {
                            break;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        sleep(Duration::from_millis(10)).await;
                        break;
                    }
                    Err(e) => {
                        error!("{}", e.to_string());
                    }
                }
            }

            // build write data
            let mut write_data_list: HashMap<SegmentIdentity, Vec<StorageEngineRecord>> =
                HashMap::new();

            let mut pkid_offset: HashMap<SegmentIdentity, HashMap<u64, u64>> = HashMap::new();

            let mut shard_sender_list: HashMap<
                SegmentIdentity,
                Vec<oneshot::Sender<SegmentWriteResp>>,
            > = HashMap::new();

            for channel_data in results {
                let shard_name = channel_data.segment_iden.shard_name.to_string();
                let segment = channel_data.segment_iden.segment;

                let shard_list = write_data_list
                    .entry(channel_data.segment_iden.clone())
                    .or_insert_with(Vec::new);
                let shard_pkid_list = pkid_offset
                    .entry(channel_data.segment_iden.clone())
                    .or_insert_with(HashMap::new);

                let sender_list = shard_sender_list
                    .entry(channel_data.segment_iden.clone())
                    .or_insert_with(Vec::new);

                match io_work.get_offset(&shard_name) {
                    Ok(offset) => {
                        let create_t = now_second();
                        let mut start_offset = offset;
                        sender_list.push(channel_data.resp_sx);
                        for row in channel_data.data_list {
                            shard_list.push(StorageEngineRecord {
                                metadata: StorageEngineRecordMetadata {
                                    offset: start_offset,
                                    shard: shard_name.clone(),
                                    segment,
                                    key: row.key,
                                    tags: row.tags,
                                    create_t: create_t.clone(),
                                },
                                data: row.value,
                            });
                            start_offset += 1;
                            shard_pkid_list.insert(row.pkid, start_offset);
                        }
                        io_work.save_offset(&shard_name, offset);
                    }
                    Err(ex) => {
                        if let Err(e) = channel_data.resp_sx.send(SegmentWriteResp {
                            error: Some(ex.to_string()),
                            ..Default::default()
                        }) {
                            error!("{:?}", e);
                        }
                    }
                }
            }

            // save data
            for (segment_iden, shard_data) in write_data_list.iter() {
                let segment_write = match open_segment_write(&cache_manager, segment_iden).await {
                    Ok(data) => data,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                };
                let pkid_offset_list = pkid_offset.get(&segment_iden).unwrap();
                match batch_write(
                    &rocksdb_engine_handler,
                    segment_iden,
                    &segment_file_manager,
                    &cache_manager,
                    &segment_write,
                    shard_data,
                    pkid_offset_list.clone(),
                )
                .await
                {
                    Ok(data) => {
                        if let Some(da) = data {
                            if let Some(sender_list) = shard_sender_list.remove(segment_iden) {
                                for sender in sender_list {
                                    if let Err(e) = sender.send(da.clone()) {
                                        error!("{:?}", e);
                                    }
                                }
                            }
                        }
                    }

                    Err(ex) => {
                        let ex_str = ex.to_string();
                        if let Some(sender_list) = shard_sender_list.remove(segment_iden) {
                            for sender in sender_list {
                                if let Err(e) = sender.send(SegmentWriteResp {
                                    error: Some(ex_str.clone()),
                                    ..Default::default()
                                }) {
                                    error!("{:?}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

/// validate whether the data can be written to the segment, write the data to the segment file and update the index
///
/// Note that this function will be executed serially by the write thread of the segment
async fn batch_write(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    segment_file_manager: &Arc<SegmentFileManager>,
    cache_manager: &Arc<StorageCacheManager>,
    segment_write: &SegmentFile,
    data_list: &Vec<StorageEngineRecord>,
    pkid_offset_list: HashMap<u64, u64>,
) -> Result<Option<SegmentWriteResp>, StorageEngineError> {
    if data_list.is_empty() {
        return Ok(None);
    }

    let resp = match segment_write.write(&data_list).await {
        Ok(_) => {
            let record = data_list.last().unwrap();
            segment_file_manager.update_end_offset(segment_iden, record.metadata.offset as i64)?;
            segment_file_manager.update_end_timestamp(segment_iden, record.metadata.create_t)?;

            Some(SegmentWriteResp {
                offsets: pkid_offset_list,
                last_offset: record.metadata.offset as u64,
                ..Default::default()
            })
        }
        Err(e) => Some(SegmentWriteResp {
            error: Some(e.to_string()),
            ..Default::default()
        }),
    };

    try_trigger_build_index(
        cache_manager,
        segment_file_manager,
        rocksdb_engine_handler,
        segment_iden,
    )
    .await?;

    Ok(resp)
}
