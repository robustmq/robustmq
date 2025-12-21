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
use metadata_struct::storage::segment::SegmentStatus;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};
use tracing::error;
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
#[derive(Default, Debug)]
pub struct SegmentWriteResp {
    pub offsets: HashMap<u64, u64>,
    pub last_offset: u64,
    pub error: Option<StorageEngineError>,
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
    io_seq: u32,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    offset_data: DashMap<String, u64>,
}

impl IoWork {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>, io_seq: u32) -> Self {
        IoWork {
            io_seq,
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
            for channel_data in results {
                let shard_name = channel_data.segment_iden.shard_name.to_string();
                let segment = channel_data.segment_iden.segment;

                let shard_list = write_data_list
                    .entry(channel_data.segment_iden.clone())
                    .or_insert_with(Vec::new);

                match io_work.get_offset(&shard_name) {
                    Ok(offset) => {
                        let create_t = now_second();
                        let mut start_offset = offset;
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
                        }
                        io_work.save_offset(&shard_name, offset);
                    }
                    Err(ex) => {
                        if let Err(e) = channel_data.resp_sx.send(SegmentWriteResp {
                            error: Some(ex),
                            ..Default::default()
                        }) {
                            error!("{:?}", e);
                        }
                    }
                }
            }

            // save data
            for (shard, shard_data) in write_data_list.iter() {
                let segment_write = match open_segment_write(&cache_manager, shard).await {
                    Ok(data) => data,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                };

                batch_write(
                    &rocksdb_engine_handler,
                    shard,
                    &segment_file_manager,
                    &cache_manager,
                    &segment_write,
                    shard_data,
                )
                .await;
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
) -> Result<Option<SegmentWriteResp>, StorageEngineError> {
    if data_list.is_empty() {
        return Ok(None);
    }

    let _offsets: Vec<u64> = data_list.iter().map(|raw| raw.metadata.offset).collect();

    let resp = match segment_write.write(&data_list).await {
        Ok(_) => {
            let record = data_list.last().unwrap();
            segment_file_manager.update_end_offset(segment_iden, record.metadata.offset as i64)?;
            segment_file_manager.update_end_timestamp(segment_iden, record.metadata.create_t)?;

            Some(SegmentWriteResp {
                offsets: HashMap::with_capacity(2),
                last_offset: record.metadata.offset as u64,
                ..Default::default()
            })
        }
        Err(e) => Some(SegmentWriteResp {
            error: Some(e),
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

fn is_end_offset(end_offset: i64, current_offset: u64, packet_len: u64) -> bool {
    end_offset > 0 && (current_offset + packet_len) > end_offset as u64
}
