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
use crate::filesegment::index::build::{save_index, BuildIndexRaw, IndexTypeEnum};
use crate::filesegment::offset::FileSegmentOffset;
use crate::filesegment::scroll::{
    is_start_or_end_offset, is_trigger_next_segment_scroll, trigger_next_segment_scroll,
    trigger_update_start_or_end_info,
};
use crate::filesegment::segment_file::open_segment_write;
use crate::filesegment::SegmentIdentity;
use bytes::Bytes;
use common_base::tools::now_second;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_read_config::AdapterWriteRespRow;
use metadata_struct::storage::storage_record::{StorageRecord, StorageRecordMetadata};
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

/// the data to be sent to the segment write thread
pub struct WriteChannelData {
    pub segment_iden: SegmentIdentity,
    pub data_list: Vec<WriteChannelDataRecord>,
    pub resp_sx: oneshot::Sender<SegmentWriteResp>,
}

#[derive(Debug, Clone)]
pub struct WriteChannelDataRecord {
    pub pkid: u64,
    pub header: Option<Vec<metadata_struct::storage::storage_record::Header>>,
    pub key: Option<String>,
    pub value: Bytes,
    pub tags: Option<Vec<String>>,
}

/// the response of the write request from the segment write thread
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
            let io_work = Arc::new(IoWork::new(
                self.rocksdb_engine_handler.clone(),
                self.cache_manager.clone(),
                i,
            ));
            create_io_thread(
                io_work,
                self.rocksdb_engine_handler.clone(),
                self.cache_manager.clone(),
                self.client_pool.clone(),
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
        if self.io_thread.is_empty() {
            return Err(StorageEngineError::NoAvailableIoThread);
        }

        let work_num = self.hash_string(&segment_iden.shard_name) % self.io_num;
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
    offset_data: DashMap<String, u64>,
    file_segment_offset: FileSegmentOffset,
}

impl IoWork {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        io_seq: u32,
    ) -> Self {
        info!("io worker {} start success", io_seq);
        IoWork {
            offset_data: DashMap::with_capacity(16),
            file_segment_offset: FileSegmentOffset::new(rocksdb_engine_handler, cache_manager),
        }
    }

    pub fn get_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(offset) = self.offset_data.get(shard_name) {
            return Ok(*offset);
        }

        let offset = self.file_segment_offset.get_latest_offset(shard_name)?;
        self.offset_data.insert(shard_name.to_string(), offset);
        Ok(offset)
    }

    pub fn save_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        self.file_segment_offset
            .save_latest_offset(segment_iden, offset)?;
        self.offset_data
            .insert(segment_iden.shard_name.to_string(), offset);
        Ok(())
    }
}

pub fn create_io_thread(
    io_work: Arc<IoWork>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
    mut data_recv: Receiver<WriteChannelData>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut stop_recv = stop_send.subscribe();
        let mut write_data_list: HashMap<SegmentIdentity, Vec<StorageRecord>> = HashMap::new();

        let mut pkid_offset: HashMap<SegmentIdentity, HashMap<u64, u64>> = HashMap::new();

        let mut shard_sender_list: HashMap<
            SegmentIdentity,
            Vec<oneshot::Sender<SegmentWriteResp>>,
        > = HashMap::new();
        let mut tmp_offset_info = HashMap::new();
        let mut index_info_list: HashMap<SegmentIdentity, Vec<BuildIndexRaw>> = HashMap::new();

        loop {
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
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive write request data from channel: {}", e);
                    }
                }
            }

            if results.is_empty() {
                sleep(Duration::from_millis(10)).await;
                continue;
            }
            write_data_list.clear();
            pkid_offset.clear();
            shard_sender_list.clear();
            tmp_offset_info.clear();
            index_info_list.clear();
            
            for channel_data in results {
                let shard_name = channel_data.segment_iden.shard_name.to_string();
                let segment = channel_data.segment_iden.segment;

                let shard_list = write_data_list
                    .entry(channel_data.segment_iden.clone())
                    .or_default();
                let shard_pkid_list = pkid_offset
                    .entry(channel_data.segment_iden.clone())
                    .or_default();

                let sender_list = shard_sender_list
                    .entry(channel_data.segment_iden.clone())
                    .or_default();
                let index_list = index_info_list
                    .entry(channel_data.segment_iden.clone())
                    .or_default();

                let offset = if let Some(offset) = tmp_offset_info.get(&shard_name) {
                    *offset
                } else {
                    let offset = match io_work.get_offset(&shard_name) {
                        Ok(offset) => offset,
                        Err(ex) => {
                            if let Err(e) = channel_data.resp_sx.send(SegmentWriteResp {
                                error: Some(ex.to_string()),
                                ..Default::default()
                            }) {
                                error!(
                                    "Failed to send get_offset error response for shard {}, segment {}: {:?}",
                                    shard_name, segment, e
                                );
                            }
                            continue;
                        }
                    };
                    offset
                };
                let create_t = now_second();
                let mut start_offset = offset;
                sender_list.push(channel_data.resp_sx);
                for row in channel_data.data_list {
                    let record_offset = start_offset;
                    shard_pkid_list.insert(row.pkid, record_offset);
                    shard_list.push(StorageRecord {
                        metadata: StorageRecordMetadata::new(
                            record_offset,
                            &shard_name,
                            segment,
                            &row.header,
                            &row.key,
                            &row.tags,
                            &row.value,
                        ),
                        data: row.value,
                    });

                    // key index
                    if let Some(key) = row.key.clone() {
                        index_list.push(BuildIndexRaw {
                            index_type: IndexTypeEnum::Key,
                            key: Some(key),
                            offset: record_offset,
                            ..Default::default()
                        });
                    }

                    // tag index
                    if let Some(tags) = row.tags.clone() {
                        for tag in tags.iter() {
                            index_list.push(BuildIndexRaw {
                                index_type: IndexTypeEnum::Tag,
                                tag: Some(tag.to_string()),
                                offset: record_offset,
                                ..Default::default()
                            });
                        }
                    }

                    // timestamp & offset index
                    if record_offset % 10000 == 0 {
                        index_list.push(BuildIndexRaw {
                            index_type: IndexTypeEnum::Time,
                            timestamp: Some(create_t),
                            offset: record_offset,
                            ..Default::default()
                        });

                        index_list.push(BuildIndexRaw {
                            index_type: IndexTypeEnum::Offset,
                            offset: record_offset,
                            ..Default::default()
                        });
                    }

                    start_offset += 1;
                }
                tmp_offset_info.insert(shard_name.to_string(), start_offset);
            }

            for (segment_iden, shard_data) in write_data_list.iter() {
                let pkid_offset_list = pkid_offset.get(segment_iden).unwrap();
                let index_list = index_info_list
                    .get(segment_iden)
                    .cloned()
                    .unwrap_or_else(Vec::new);

                match batch_write(
                    &cache_manager,
                    &rocksdb_engine_handler,
                    &client_pool,
                    segment_iden,
                    shard_data,
                    pkid_offset_list,
                    &index_list,
                )
                .await
                {
                    Ok(data) => {
                        if let Some(resp) = data {
                            if success_save_offset(
                                &mut shard_sender_list,
                                pkid_offset_list,
                                &io_work,
                                segment_iden,
                            ) {
                                call_success_response(&mut shard_sender_list, segment_iden, &resp);
                            }
                        }
                    }

                    Err(ex) => {
                        call_error_response(&mut shard_sender_list, segment_iden, &ex.to_string());
                    }
                }
            }
        }
    });
}

fn success_save_offset(
    shard_sender_list: &mut HashMap<SegmentIdentity, Vec<oneshot::Sender<SegmentWriteResp>>>,
    pkid_offset_list: &HashMap<u64, u64>,
    io_work: &Arc<IoWork>,
    segment_iden: &SegmentIdentity,
) -> bool {
    if let Some(max_offset) = pkid_offset_list.values().max() {
        if let Err(ex) = io_work.save_offset(segment_iden, *max_offset + 1) {
            call_error_response(shard_sender_list, segment_iden, &ex.to_string());
            return false;
        }
    }
    true
}

fn call_success_response(
    shard_sender_list: &mut HashMap<SegmentIdentity, Vec<oneshot::Sender<SegmentWriteResp>>>,
    segment_iden: &SegmentIdentity,
    resp: &SegmentWriteResp,
) {
    if let Some(sender_list) = shard_sender_list.remove(segment_iden) {
        for sender in sender_list {
            if let Err(e) = sender.send(resp.clone()) {
                error!(
                    "Failed to send success response to client for shard {}, segment {}: {:?}",
                    segment_iden.shard_name, segment_iden.segment, e
                );
            }
        }
    }
}

fn call_error_response(
    shard_sender_list: &mut HashMap<SegmentIdentity, Vec<oneshot::Sender<SegmentWriteResp>>>,
    segment_iden: &SegmentIdentity,
    ex_str: &str,
) {
    if let Some(sender_list) = shard_sender_list.remove(segment_iden) {
        for sender in sender_list {
            if let Err(e) = sender.send(SegmentWriteResp {
                error: Some(ex_str.to_string()),
                ..Default::default()
            }) {
                error!(
                    "Failed to send error response to client for shard {}, segment {}: {:?}",
                    segment_iden.shard_name, segment_iden.segment, e
                );
            }
        }
    }
}

/// validate whether the data can be written to the segment, write the data to the segment file and update the index
///
/// Note that this function will be executed serially by the write thread of the segment
async fn batch_write(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    data_list: &[StorageRecord],
    pkid_offset_list: &HashMap<u64, u64>,
    index_data: &[BuildIndexRaw],
) -> Result<Option<SegmentWriteResp>, StorageEngineError> {
    if data_list.is_empty() {
        return Ok(None);
    }

    let offsets: Vec<u64> = data_list.iter().map(|raw| raw.metadata.offset).collect();
    let last_offset = offsets.iter().max().unwrap();

    if !cache_manager
        .segment_file_writer
        .contains_key(&segment_iden.name())
    {
        let segment_file = open_segment_write(cache_manager, segment_iden).await?;
        cache_manager.add_segment_file_write(segment_iden, segment_file);
    }

    let mut segment_write = if let Some(segment_file) = cache_manager
        .segment_file_writer
        .get_mut(&segment_iden.name())
    {
        segment_file
    } else {
        return Err(StorageEngineError::ReadSegmentFileError(
            segment_iden.name(),
        ));
    };

    let offset_positions = segment_write.write(data_list).await?;

    // save index
    save_index(
        rocksdb_engine_handler,
        segment_iden,
        index_data,
        &offset_positions,
    )?;

    // trigger scroll next segment
    if is_trigger_next_segment_scroll(&offsets) {
        if let Err(e) = trigger_next_segment_scroll(
            cache_manager,
            client_pool,
            &segment_write,
            segment_iden,
            *last_offset,
        )
        .await
        {
            error!("{}", e);
        }
    }

    // trigger start/end info update
    if is_start_or_end_offset(cache_manager, segment_iden, &offsets) {
        trigger_update_start_or_end_info(
            cache_manager.clone(),
            client_pool.clone(),
            segment_iden.clone(),
            offsets.clone(),
        );
    }

    let offsets: Vec<AdapterWriteRespRow> = pkid_offset_list
        .iter()
        .map(|raw| AdapterWriteRespRow {
            pkid: *raw.0,
            offset: *raw.1,
            ..Default::default()
        })
        .collect();

    Ok(Some(SegmentWriteResp {
        offsets,
        last_offset: *last_offset,
        ..Default::default()
    }))
}

// async fn is_write_next_segment(last_offset: u64) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_init_segment;
    use crate::filesegment::segment_file::SegmentFile;
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use metadata_struct::storage::storage_record::StorageRecordMetadata;

    fn create_test_records(count: usize, shard: &str, segment: u32) -> Vec<StorageRecord> {
        (0..count)
            .map(|i| {
                let data = Bytes::from(format!("data-{}", i));
                StorageRecord {
                    metadata: StorageRecordMetadata::new(
                        i as u64,
                        shard,
                        segment,
                        &None,
                        &Some(format!("key-{}", i)),
                        &Some(vec![format!("tag-{}", i)]),
                        &data,
                    ),
                    data,
                }
            })
            .collect()
    }

    #[tokio::test]
    async fn batch_write_test() {
        let (segment_iden, cache_manager, fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let client_poll = Arc::new(ClientPool::new(100));

        let test_records = create_test_records(5, &segment_iden.shard_name, segment_iden.segment);
        let mut pkid_offset = HashMap::new();
        for (i, _) in test_records.iter().enumerate() {
            pkid_offset.insert(i as u64, i as u64);
        }
        let index_data = Vec::new();

        let result = batch_write(
            &cache_manager,
            &rocksdb,
            &client_poll,
            &segment_iden,
            &test_records,
            &pkid_offset,
            &index_data,
        )
        .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.is_some());

        let resp_data = resp.unwrap();
        assert_eq!(resp_data.offsets.len(), 5);
        assert_eq!(resp_data.last_offset, 4);

        let read_result = segment_file.read_by_offset(0, 0, 1024 * 1024, 100).await;
        assert!(read_result.is_ok());
        let read_records = read_result.unwrap();
        assert_eq!(read_records.len(), 5);
        for (i, record) in read_records.iter().enumerate() {
            assert_eq!(record.record.metadata.offset, i as u64);
            assert_eq!(record.record.data, Bytes::from(format!("data-{}", i)));
        }
    }

    #[tokio::test]
    async fn batch_write_empty_data_test() {
        let (segment_iden, cache_manager, fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let empty_records: Vec<StorageRecord> = vec![];
        let pkid_offset = HashMap::new();
        let client_poll = Arc::new(ClientPool::new(100));
        let index_data = Vec::new();

        let result = batch_write(
            &cache_manager,
            &rocksdb,
            &client_poll,
            &segment_iden,
            &empty_records,
            &pkid_offset,
            &index_data,
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let read_result = segment_file.read_by_offset(0, 0, 1024 * 1024, 100).await;
        assert!(read_result.is_ok());
        let read_records = read_result.unwrap();
        assert_eq!(read_records.len(), 0);
    }

    #[tokio::test]
    async fn write_manager_write_test() {
        let (segment_iden, cache_manager, fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let client_poll = Arc::new(ClientPool::new(100));

        let write_manager =
            WriteManager::new(rocksdb.clone(), cache_manager.clone(), client_poll, 3);

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut data_list = Vec::new();
        for i in 0..10 {
            data_list.push(WriteChannelDataRecord {
                pkid: i,
                header: None,
                key: Some(format!("key-{}", i)),
                tags: Some(vec![format!("tag-{}", i)]),
                value: Bytes::from(format!("data-{}", i)),
            });
        }

        let result = write_manager.write(&segment_iden, data_list).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        if let Some(ref err) = resp.error {
            panic!("Write failed with error: {}", err);
        }
        assert_eq!(resp.offsets.len(), 10);
        assert_eq!(resp.last_offset, 9);

        for row in &resp.offsets {
            assert_eq!(row.pkid, row.offset);
        }

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let read_result = segment_file.read_by_offset(0, 0, 1024 * 1024, 100).await;
        assert!(read_result.is_ok());
        let read_records = read_result.unwrap();
        assert_eq!(read_records.len(), 10);
        for (i, record) in read_records.iter().enumerate() {
            assert_eq!(record.record.metadata.offset, i as u64);
            assert_eq!(record.record.data, Bytes::from(format!("data-{}", i)));
        }
    }

    #[tokio::test]
    async fn write_manager_segment_not_exist_error_test() {
        let (_, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let client_poll = Arc::new(ClientPool::new(100));

        let non_exist_segment = SegmentIdentity {
            shard_name: "non_exist_shard".to_string(),
            segment: 999,
        };

        let write_manager = WriteManager::new(
            rocksdb.clone(),
            cache_manager.clone(),
            client_poll.clone(),
            3,
        );

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let data_list = vec![WriteChannelDataRecord {
            pkid: 1,
            header: None,
            key: Some("key-1".to_string()),
            tags: None,
            value: Bytes::from("data-1"),
        }];

        let result = write_manager.write(&non_exist_segment, data_list).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert!(resp.error.is_some());
    }

    #[tokio::test]
    async fn write_manager_no_io_thread_error_test() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let client_poll = Arc::new(ClientPool::new(100));

        let write_manager = WriteManager::new(
            rocksdb.clone(),
            cache_manager.clone(),
            client_poll.clone(),
            3,
        );

        let data_list = vec![WriteChannelDataRecord {
            pkid: 1,
            header: None,
            key: Some("key-1".to_string()),
            tags: None,
            value: Bytes::from("data-1"),
        }];

        let result = write_manager.write(&segment_iden, data_list).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageEngineError::NoAvailableIoThread
        ));
    }
}
