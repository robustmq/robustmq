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
use std::sync::Arc;
use std::time::Duration;

use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use log::error;
use metadata_struct::journal::segment::SegmentStatus;
use protocol::journal_server::journal_engine::{
    WriteReqBody, WriteRespMessage, WriteRespMessageStatus,
};
use protocol::journal_server::journal_record::JournalRecord;
use rocksdb_engine::RocksDBEngine;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::segment_status::sealup_segment;
use crate::index::build::try_trigger_build_index;

use crate::segment::file::{open_segment_write, SegmentFile};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct SegmentWrite {
    data_sender: Sender<SegmentWriteData>,
    pub stop_sender: broadcast::Sender<bool>,
}

pub struct SegmentWriteData {
    data: Vec<JournalRecord>,
    resp_sx: oneshot::Sender<SegmentWriteResp>,
}

#[derive(Default, Debug)]
pub struct SegmentWriteResp {
    offsets: HashMap<u64, u64>,
    last_offset: u64,
    error: Option<JournalServerError>,
}

pub async fn write_data_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    req_body: &WriteReqBody,
) -> Result<Vec<WriteRespMessage>, JournalServerError> {
    let mut results = Vec::new();
    for shard_data in req_body.data.clone() {
        let mut resp_message = WriteRespMessage {
            namespace: shard_data.namespace.clone(),
            shard_name: shard_data.shard_name.clone(),
            segment: shard_data.segment,
            ..Default::default()
        };

        let segment_iden = SegmentIdentity::new(
            &shard_data.namespace,
            &shard_data.shard_name,
            shard_data.segment,
        );

        let mut data_list = Vec::new();
        for message in shard_data.messages.iter() {
            // todo data validator
            let record = JournalRecord {
                content: message.value.clone(),
                create_time: now_second(),
                key: message.key.clone(),
                namespace: shard_data.namespace.clone(),
                shard_name: shard_data.shard_name.clone(),
                segment: shard_data.segment,
                tags: message.tags.clone(),
                pkid: message.pkid,
                producer_id: "".to_string(),
                offset: -1,
            };
            data_list.push(record);
        }

        let resp = write_data(
            cache_manager,
            rocksdb_engine_handler,
            segment_file_manager,
            client_pool,
            &segment_iden,
            data_list.clone(),
        )
        .await?;

        if let Some(e) = resp.error {
            return Err(e);
        }

        let last_record = data_list.last().unwrap();
        segment_file_manager.update_end_offset(&segment_iden, last_record.offset)?;
        segment_file_manager.update_end_timestamp(&segment_iden, last_record.create_time)?;

        let mut resp_message_status = Vec::new();
        for (pkid, offset) in resp.offsets {
            let status = WriteRespMessageStatus {
                pkid,
                offset,
                ..Default::default()
            };
            resp_message_status.push(status);
        }
        resp_message.messages = resp_message_status;
        results.push(resp_message);
    }
    Ok(results)
}

pub(crate) async fn write_data(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    data_list: Vec<JournalRecord>,
) -> Result<SegmentWriteResp, JournalServerError> {
    let write = get_write(
        cache_manager,
        rocksdb_engine_handler,
        segment_file_manager,
        client_pool,
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

async fn get_write(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<SegmentWrite, JournalServerError> {
    let write = if let Some(write) = cache_manager.get_segment_write_thread(segment_iden) {
        write.clone()
    } else {
        create_write_thread(
            cache_manager,
            rocksdb_engine_handler,
            segment_file_manager,
            client_pool,
            segment_iden,
        )
        .await?
    };
    Ok(write)
}

pub(crate) async fn create_write_thread(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
) -> Result<SegmentWrite, JournalServerError> {
    let (data_sender, data_recv) = mpsc::channel::<SegmentWriteData>(1000);
    let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

    let segment_file_meta =
        if let Some(segment_file) = segment_file_manager.get_segment_file(segment_iden) {
            segment_file
        } else {
            // todo try recover segment file. create or local cache
            return Err(JournalServerError::SegmentFileMetaNotExists(
                segment_iden.name(),
            ));
        };

    let (segment_write, max_file_size) = open_segment_write(cache_manager, segment_iden).await?;

    create_write_thread0(
        rocksdb_engine_handler.clone(),
        segment_iden.clone(),
        segment_file_manager.clone(),
        cache_manager.clone(),
        client_pool.clone(),
        segment_file_meta.end_offset,
        segment_write,
        max_file_size,
        data_recv,
        stop_recv,
    )
    .await;

    let write = SegmentWrite {
        data_sender,
        stop_sender,
    };
    cache_manager.add_segment_write_thread(segment_iden, write.clone());
    Ok(write)
}

#[allow(clippy::too_many_arguments)]
async fn create_write_thread0(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_iden: SegmentIdentity,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    mut local_segment_end_offset: i64,
    segment_write: SegmentFile,
    max_file_size: u32,
    mut data_recv: Receiver<SegmentWriteData>,
    mut stop_recv: broadcast::Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            cache_manager.remove_segment_write_thread(&segment_iden);
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
                        &rocksdb_engine_handler,
                        &segment_iden,
                        &segment_file_manager,
                        &cache_manager,
                        &client_pool,
                        local_segment_end_offset,
                        &segment_write,
                        max_file_size,
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

#[allow(clippy::too_many_arguments)]
async fn batch_write(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    segment_file_manager: &Arc<SegmentFileManager>,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    local_segment_end_offset: i64,
    segment_write: &SegmentFile,
    max_file_size: u32,
    data: Vec<JournalRecord>,
) -> Result<Option<SegmentWriteResp>, JournalServerError> {
    if data.is_empty() {
        return Ok(None);
    }

    let flag = is_sealup_segment(
        cache_manager,
        client_pool,
        segment_write,
        local_segment_end_offset as u64,
        data.len() as u64,
        max_file_size,
    )
    .await?;

    if flag {
        return Ok(Some(SegmentWriteResp {
            error: Some(JournalServerError::SegmentAlreadySealUp(
                segment_iden.name(),
            )),
            ..Default::default()
        }));
    }

    let resp = batch_write0(data, segment_write, local_segment_end_offset as u64).await?;

    try_trigger_build_index(
        cache_manager,
        segment_file_manager,
        rocksdb_engine_handler,
        segment_iden,
    )
    .await?;

    Ok(resp)
}

async fn batch_write0(
    data: Vec<JournalRecord>,
    segment_write: &SegmentFile,
    mut local_segment_end_offset: u64,
) -> Result<Option<SegmentWriteResp>, JournalServerError> {
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

async fn is_sealup_segment(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_write: &SegmentFile,
    local_segment_end_offset: u64,
    packet_len: u64,
    max_file_size: u32,
) -> Result<bool, JournalServerError> {
    let segment_iden = SegmentIdentity::new(
        &segment_write.namespace,
        &segment_write.shard_name,
        segment_write.segment_no,
    );

    let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
        segment
    } else {
        return Err(JournalServerError::SegmentNotExist(segment_iden.name()));
    };

    if segment.status == SegmentStatus::SealUp {
        return Err(JournalServerError::SegmentAlreadySealUp(
            segment_iden.name(),
        ));
    }

    let segment_meta = if let Some(meta) = cache_manager.get_segment_meta(&segment_iden) {
        meta
    } else {
        return Err(JournalServerError::SegmentMetaNotExists(
            segment_iden.name(),
        ));
    };

    let file_size = (segment_write.size().await).unwrap_or_default();

    if is_sealup_segment0(
        segment_meta.end_offset,
        local_segment_end_offset,
        packet_len,
        file_size,
        max_file_size as u64,
    ) {
        sealup_segment(cache_manager, client_pool, &segment_iden).await?;
        return Ok(true);
    }
    Ok(false)
}

fn is_sealup_segment0(
    end_offset: i64,
    current_offset: u64,
    packet_len: u64,
    file_size: u64,
    max_file_size: u64,
) -> bool {
    (end_offset > 0 && (current_offset + packet_len) > end_offset as u64)
        || file_size >= max_file_size
}

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use prost::Message;
    use protocol::journal_server::journal_record::JournalRecord;

    use super::{create_write_thread, is_sealup_segment0, write_data};
    use crate::core::test::{test_init_client_pool, test_init_segment};
    use crate::segment::file::open_segment_write;

    #[tokio::test]
    async fn is_sealup_segment_test() {
        let mut end_offset = -1;
        let current_offset = 0;
        let packet_len = 3;

        let mut file_size = 100;
        let max_file_size = 300;
        assert!(!is_sealup_segment0(
            end_offset,
            current_offset,
            packet_len,
            file_size,
            max_file_size
        ));

        file_size = 301;
        assert!(is_sealup_segment0(
            end_offset,
            current_offset,
            packet_len,
            file_size,
            max_file_size
        ));

        end_offset = 2;
        file_size = 30;
        assert!(is_sealup_segment0(
            end_offset,
            current_offset,
            packet_len,
            file_size,
            max_file_size
        ));
    }

    #[tokio::test]
    async fn write_test() {
        let (segment_iden, cache_manager, segment_file_manager, _, rocksdb_engine_handler) =
            test_init_segment().await;
        let client_pool = test_init_client_pool();

        let res = create_write_thread(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &client_pool,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        let mut data_list = Vec::new();

        let producer_id = unique_id();
        for i in 0..10 {
            data_list.push(JournalRecord {
                namespace: segment_iden.namespace.clone(),
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                content: format!("data-{}", i).encode_to_vec(),
                pkid: i,
                producer_id: producer_id.clone(),
                ..Default::default()
            });
        }

        let res = write_data(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &client_pool,
            &segment_iden,
            data_list,
        )
        .await;

        println!("{:?}", res);
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.offsets.len(), 10);
        assert_eq!(resp.last_offset, 9);

        let mut data_list = Vec::new();
        for i in 10..20 {
            data_list.push(JournalRecord {
                namespace: segment_iden.namespace.clone(),
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                content: format!("data-{}", i).encode_to_vec(),
                pkid: i,
                producer_id: producer_id.clone(),
                ..Default::default()
            });
        }

        let res = write_data(
            &cache_manager,
            &rocksdb_engine_handler,
            &segment_file_manager,
            &client_pool,
            &segment_iden,
            data_list,
        )
        .await;

        println!("{:?}", res);
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.offsets.len(), 10);
        assert_eq!(resp.last_offset, 19);

        let write = open_segment_write(&cache_manager, &segment_iden)
            .await
            .unwrap();
        let res = write.0.read_by_offset(0, 0, 1024 * 1024 * 1024, 1000).await;
        assert!(res.is_ok());

        let resp = res.unwrap();
        assert_eq!(resp.len(), 20);

        for (i, row) in resp.into_iter().enumerate() {
            assert_eq!(i, row.record.offset as usize);
        }
    }
}
