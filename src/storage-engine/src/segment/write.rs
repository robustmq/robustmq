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
use crate::segment::file::{open_segment_write, SegmentFile};
use crate::segment::index::build::try_trigger_build_index;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;
use metadata_struct::storage::segment::SegmentStatus;
use protocol::storage::storage_engine_record::StorageEngineRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};
use tracing::error;

/// the write handle for a segment
#[derive(Clone)]
pub struct SegmentWrite {
    pub data_sender: Sender<SegmentWriteData>,
    pub stop_sender: broadcast::Sender<bool>,
}

/// the data to be sent to the segment write thread
pub struct SegmentWriteData {
    data: Vec<StorageEngineRecord>,
    resp_sx: oneshot::Sender<SegmentWriteResp>,
}

/// the response of the write request from the segment write thread
#[derive(Default, Debug)]
pub struct SegmentWriteResp {
    pub offsets: HashMap<u64, u64>,
    pub last_offset: u64,
    pub error: Option<StorageEngineError>,
}

/// get the write handle for the segment identified by `segment_iden`, write data and return the response
pub(crate) async fn write_data(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
    data_list: Vec<StorageEngineRecord>,
) -> Result<SegmentWriteResp, StorageEngineError> {
    let write = get_write(
        cache_manager,
        rocksdb_engine_handler,
        segment_file_manager,
        segment_iden,
    )
    .await?;

    let (sx, rx) = oneshot::channel::<SegmentWriteResp>();
    let data = SegmentWriteData {
        data: data_list,
        resp_sx: sx,
    };
    write.data_sender.send(data).await?;

    let time_res: Result<SegmentWriteResp, oneshot::error::RecvError> =
        timeout(Duration::from_secs(30), rx).await?;
    Ok(time_res?)
}

/// get the write handle for the segment identified by `segment_iden`
///
/// If the write handle does not exist, create a new one
///
/// Note that this function may be executed concurrently by multiple threads
///
/// TODO: maybe we should use [`DashMap::entry()`](https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html#method.entry)
/// with [`or_insert`](https://docs.rs/dashmap/latest/dashmap/mapref/entry/enum.Entry.html#method.or_insert) to prevent creating multiple handles for the same segment
pub async fn get_write(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
) -> Result<SegmentWrite, StorageEngineError> {
    let write = if let Some(write) = cache_manager.get_segment_write_thread(segment_iden) {
        write
    } else {
        create_write_thread(
            cache_manager,
            rocksdb_engine_handler,
            segment_file_manager,
            segment_iden,
        )
        .await?
    };
    Ok(write)
}

/// create a segment write thread which is responsible for writing data to the segment file identified by `segment_iden`
///
/// Return a `SegmentWrite` handle which can be used to send data to the write thread
pub(crate) async fn create_write_thread(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
) -> Result<SegmentWrite, StorageEngineError> {
    let (data_sender, data_recv) = mpsc::channel::<SegmentWriteData>(1000);
    let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

    let segment_file_meta =
        if let Some(segment_file) = segment_file_manager.get_segment_file(segment_iden) {
            segment_file
        } else {
            // todo try recover segment file. create or local cache
            return Err(StorageEngineError::SegmentFileMetaNotExists(
                segment_iden.name(),
            ));
        };

    let segment_write = open_segment_write(cache_manager, segment_iden).await?;

    let context = WriteThreadContext {
        rocksdb_engine_handler: rocksdb_engine_handler.clone(),
        segment_iden: segment_iden.clone(),
        segment_file_manager: segment_file_manager.clone(),
        cache_manager: cache_manager.clone(),
        local_segment_end_offset: segment_file_meta.end_offset,
        segment_write,
    };

    create_write_thread0(context, data_recv, stop_recv).await;

    let write = SegmentWrite {
        data_sender,
        stop_sender,
    };
    cache_manager.add_segment_write_thread(segment_iden, write.clone());
    Ok(write)
}

pub struct WriteThreadContext {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub segment_iden: SegmentIdentity,
    pub segment_file_manager: Arc<SegmentFileManager>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub local_segment_end_offset: i64,
    pub segment_write: SegmentFile,
}

/// spawn the write thread for a segment
async fn create_write_thread0(
    context: WriteThreadContext,
    mut data_recv: Receiver<SegmentWriteData>,
    mut stop_recv: broadcast::Receiver<bool>,
) {
    let mut local_segment_end_offset = context.local_segment_end_offset;
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            context.cache_manager.remove_segment_write_thread(&context.segment_iden);
                            break;
                        }
                    }
                },
                val = data_recv.recv()=>{

                    if val.is_none(){
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }

                    let packet = val.unwrap();

                    let resp = match batch_write(
                        &context.rocksdb_engine_handler,
                        &context.segment_iden,
                        &context.segment_file_manager,
                        &context.cache_manager,
                        local_segment_end_offset,
                        &context.segment_write,
                        packet.data
                    ).await{
                        Ok(Some(resp)) => {
                            local_segment_end_offset = resp.last_offset as i64;
                            resp
                        },
                        Ok(None) =>{
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        },
                        Err(e) => {
                            SegmentWriteResp {
                                error: Some(e),
                                ..Default::default()
                            }
                        }
                    };


                    if packet.resp_sx.send(resp).is_err(){
                        error!("Write data to the Segment file, write success, call the oneshot channel to return the write information failed. Failure message");
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
    local_segment_end_offset: i64,
    segment_write: &SegmentFile,
    data: Vec<StorageEngineRecord>,
) -> Result<Option<SegmentWriteResp>, StorageEngineError> {
    if data.is_empty() {
        return Ok(None);
    }

    write_validator(
        cache_manager,
        segment_write,
        local_segment_end_offset as u64,
        data.len() as u64,
    )
    .await?;

    let resp = batch_write0(
        data,
        segment_write,
        segment_file_manager,
        segment_iden,
        local_segment_end_offset as u64,
    )
    .await?;

    try_trigger_build_index(
        cache_manager,
        segment_file_manager,
        rocksdb_engine_handler,
        segment_iden,
    )
    .await?;

    Ok(resp)
}

/// write a batch of data to the segment file
///
/// Note that this function will be executed serially by the write thread of the segment
async fn batch_write0(
    data: Vec<StorageEngineRecord>,
    segment_write: &SegmentFile,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
    mut local_segment_end_offset: u64,
) -> Result<Option<SegmentWriteResp>, StorageEngineError> {
    if data.is_empty() {
        return Ok(None);
    }

    // build write data
    let mut offsets = HashMap::new();
    let mut records = Vec::new();
    for mut record in data.clone() {
        let offset = local_segment_end_offset + 1;
        record.offset = offset as i64;
        records.push(record.clone());

        offsets.insert(record.pkid, offset);
        local_segment_end_offset = offset;
    }

    // batch write data
    match segment_write.write(&records).await {
        Ok(_) => {
            let record = records.last().unwrap();
            segment_file_manager.update_end_offset(segment_iden, record.offset)?;
            segment_file_manager.update_end_timestamp(segment_iden, record.create_time)?;

            Ok(Some(SegmentWriteResp {
                offsets: offsets.clone(),
                last_offset: record.offset as u64,
                ..Default::default()
            }))
        }
        Err(e) => Ok(Some(SegmentWriteResp {
            error: Some(e),
            ..Default::default()
        })),
    }
}

/// validate whether the data can be written to the segment
///
/// Note that this function will be executed serially by the write thread of the segment
async fn write_validator(
    cache_manager: &Arc<StorageCacheManager>,
    segment_write: &SegmentFile,
    local_segment_end_offset: u64,
    packet_len: u64,
) -> Result<(), StorageEngineError> {
    let segment_iden = SegmentIdentity::new(&segment_write.shard_name, segment_write.segment_no);

    let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
        segment
    } else {
        return Err(StorageEngineError::SegmentNotExist(segment_iden.name()));
    };

    if segment.status == SegmentStatus::SealUp {
        return Err(StorageEngineError::SegmentAlreadySealUp(
            segment_iden.name(),
        ));
    }

    let segment_meta = if let Some(meta) = cache_manager.get_segment_meta(&segment_iden) {
        meta
    } else {
        return Err(StorageEngineError::SegmentMetaNotExists(
            segment_iden.name(),
        ));
    };

    if is_end_offset(
        segment_meta.end_offset,
        local_segment_end_offset,
        packet_len,
    ) {
        cache_manager.update_segment_status(&segment_iden, SegmentStatus::SealUp);
        return Err(StorageEngineError::SegmentOffsetAtTheEnd);
    }
    Ok(())
}

fn is_end_offset(end_offset: i64, current_offset: u64, packet_len: u64) -> bool {
    end_offset > 0 && (current_offset + packet_len) > end_offset as u64
}

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use prost::Message;
    use protocol::storage::storage_engine_record::StorageEngineRecord;

    use super::{create_write_thread, is_end_offset, write_data};
    use crate::core::test::test_init_segment;
    use crate::segment::file::open_segment_write;

    #[tokio::test]
    async fn is_sealup_segment_test() {
        let mut end_offset = -1;
        let current_offset = 0;
        let packet_len = 3;

        assert!(!is_end_offset(end_offset, current_offset, packet_len));

        end_offset = 2;
        assert!(is_end_offset(end_offset, current_offset, packet_len));

        end_offset = 4;
        assert!(!is_end_offset(end_offset, current_offset, packet_len));
    }

    #[tokio::test]
    async fn write_test() {
        let (segment_iden, cache_manager, segment_file_manager, _, rocksdb_engine_handler) =
            test_init_segment().await;

        let res = create_write_thread(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        let mut data_list = Vec::new();

        let producer_id = unique_id();
        for i in 0..10 {
            data_list.push(StorageEngineRecord {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                content: format!("data-{i}").encode_to_vec(),
                pkid: i,
                producer_id: producer_id.clone(),
                ..Default::default()
            });
        }

        let res = write_data(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &segment_iden,
            data_list,
        )
        .await;

        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.offsets.len(), 10);
        assert_eq!(resp.last_offset, 9);

        let mut data_list = Vec::new();
        for i in 10..20 {
            data_list.push(StorageEngineRecord {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                content: format!("data-{i}").encode_to_vec(),
                pkid: i,
                producer_id: producer_id.clone(),
                ..Default::default()
            });
        }

        let res = write_data(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &segment_iden,
            data_list,
        )
        .await;

        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.offsets.len(), 10);
        assert_eq!(resp.last_offset, 19);

        let write = open_segment_write(&cache_manager, &segment_iden)
            .await
            .unwrap();
        let res = write.read_by_offset(0, 0, 1024 * 1024 * 1024, 1000).await;
        assert!(res.is_ok());

        let resp = res.unwrap();
        assert_eq!(resp.len(), 20);

        for (i, row) in resp.into_iter().enumerate() {
            assert_eq!(i, row.record.offset as usize);
        }
    }
}
