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
use tokio::time::timeout;

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::segment_meta::{update_meta_end_timestamp, update_meta_start_timestamp};
use crate::core::segment_status::sealup_segment;
use crate::index::build::try_trigger_build_index;
use crate::segment::file::{open_segment_write, SegmentFile};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct SegmentWrite {
    last_write_time: u64,
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
    positions: HashMap<u64, u64>,
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
                ..Default::default()
            };
            data_list.push(record);
        }

        let resp = write(
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

        // if position = 0, update start/timestamp
        for (_, position) in resp.positions.iter() {
            if *position == 0 {
                let first_record = data_list.first().unwrap();
                segment_position0_ac(
                    segment_file_manager,
                    client_pool,
                    &segment_iden,
                    *position as i64,
                    first_record.create_time,
                )
                .await?;
            }
        }

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

async fn write(
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
    let key = segment_iden.name();
    let write = if let Some(write) = cache_manager.get_segment_write_thread(segment_iden) {
        write.clone()
    } else {
        let (data_sender, data_recv) = mpsc::channel::<SegmentWriteData>(1000);
        let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);
        start_segment_sync_write_thread(
            rocksdb_engine_handler.clone(),
            segment_iden.clone(),
            segment_file_manager.clone(),
            cache_manager.clone(),
            client_pool.clone(),
            data_recv,
            stop_recv,
        )
        .await?;

        let write = SegmentWrite {
            last_write_time: 0,
            data_sender,
            stop_sender,
        };
        cache_manager.add_segment_write_thread(segment_iden, write.clone());
        write
    };
    Ok(write)
}

async fn start_segment_sync_write_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_iden: SegmentIdentity,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    mut data_recv: Receiver<SegmentWriteData>,
    mut stop_recv: broadcast::Receiver<bool>,
) -> Result<(), JournalServerError> {
    let segment_file_meta =
        if let Some(segment_file) = segment_file_manager.get_segment_file(&segment_iden) {
            segment_file
        } else {
            // todo try recover segment file. create or local cache
            return Err(JournalServerError::SegmentMetaNotExists(
                segment_iden.name(),
            ));
        };

    let mut local_segment_end_offset = segment_file_meta.end_offset;
    let (segment_write, max_file_size) = open_segment_write(&cache_manager, &segment_iden).await?;

    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            break;
                        }
                    }
                },
                val = data_recv.recv()=>{
                    if let Some(packet) = val{
                        if packet.data.is_empty(){
                            continue;
                        }

                        let mut resp = SegmentWriteResp::default();
                        let mut is_break = false;
                        match is_sealup_segment(
                            &cache_manager,
                            &client_pool,
                            &segment_write,
                            local_segment_end_offset as u64,
                            packet.data.len() as u64,
                            max_file_size,
                        ).await{
                            Ok(sealup) => {
                                if sealup {
                                    resp.error = Some(JournalServerError::SegmentAlreadySealUp(segment_iden.name()));
                                    is_break = true;
                                }else{
                                    match batch_write_segment(
                                        &packet,
                                        &segment_write,
                                        &segment_file_manager,
                                        &client_pool,
                                        local_segment_end_offset as u64).await
                                    {
                                        Ok((resp_data,last_offset)) =>{
                                            if let Some(end_offset) = last_offset {
                                                local_segment_end_offset = end_offset as i64;
                                            }
                                            resp = resp_data;

                                            try_trigger_build_index(&cache_manager, &segment_file_manager, &rocksdb_engine_handler, &segment_iden).await;
                                        },
                                        Err(e) => {
                                            resp.error = Some(e);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                resp.error = Some(e);
                            }
                        }
                        // resp write client
                        if packet.resp_sx.send(resp).is_err(){
                            error!("Write data to the Segment file, write success, call the oneshot channel to return the write information failed. Failure message");
                        }

                        if is_break{
                            cache_manager.remove_segment_write_thread(&segment_iden);
                            break;
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

async fn batch_write_segment(
    packet: &SegmentWriteData,
    segment_write: &SegmentFile,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    mut local_segment_end_offset: u64,
) -> Result<(SegmentWriteResp, Option<u64>), JournalServerError> {
    let segment_iden = SegmentIdentity {
        namespace: segment_write.namespace.clone(),
        shard_name: segment_write.shard_name.clone(),
        segment_seq: segment_write.segment_no,
    };

    let mut resp = SegmentWriteResp::default();

    // build write data
    let mut last_offset = None;
    let mut offsets = HashMap::new();
    let mut records = Vec::new();
    for mut record in packet.data.clone() {
        record.offset = local_segment_end_offset + 1;
        local_segment_end_offset = record.offset;

        offsets.insert(record.pkid, record.offset);
        last_offset = Some(record.offset);

        records.push(record);
    }

    // batch write data
    match segment_write.write(&records).await {
        Ok(positions) => {
            resp.offsets = offsets.clone();
            resp.positions = positions;
        }
        Err(e) => {
            resp.error = Some(e);
        }
    }

    // update local segment file end offset/Timestamp
    if let Some(end_offset) = last_offset {
        let last_recrd = records.last().unwrap();
        segment_position9_ac(
            client_pool,
            segment_file_manager,
            &segment_iden,
            end_offset as i64,
            last_recrd.create_time,
        )
        .await?;
    }

    Ok((resp, last_offset))
}

async fn segment_position0_ac(
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    segment_iden: &SegmentIdentity,
    first_posi: i64,
    timestamp: u64,
) -> Result<(), JournalServerError> {
    // update local file start offset
    segment_file_manager.update_start_offset(segment_iden, first_posi)?;

    // update local file start timestamp
    segment_file_manager.update_start_timestamp(segment_iden, timestamp)?;

    // update meta start timestamp
    update_meta_start_timestamp(client_pool.clone(), segment_iden, timestamp).await?;
    Ok(())
}

async fn segment_position9_ac(
    client_pool: &Arc<ClientPool>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
    end_offset: i64,
    timestamp: u64,
) -> Result<(), JournalServerError> {
    // update local file end offset
    segment_file_manager.update_end_offset(segment_iden, end_offset)?;

    // update local file end timestamp
    segment_file_manager.update_end_timestamp(segment_iden, timestamp)?;

    // update meta end timestamp
    update_meta_end_timestamp(client_pool.clone(), segment_iden, timestamp).await?;
    Ok(())
}

async fn is_sealup_segment(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_write: &SegmentFile,
    local_segment_end_offset: u64,
    packet_len: u64,
    max_file_size: u64,
) -> Result<bool, JournalServerError> {
    let segment_iden = SegmentIdentity {
        namespace: segment_write.namespace.to_string(),
        shard_name: segment_write.shard_name.to_string(),
        segment_seq: segment_write.segment_no,
    };

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

    // check if the offset exceeds the end offset
    let segment_meta = if let Some(meta) = cache_manager.get_segment_meta(&segment_iden) {
        meta
    } else {
        return Err(JournalServerError::SegmentMetaNotExists(
            segment_iden.name(),
        ));
    };

    // check file size
    let file_size = (segment_write.size().await).unwrap_or_default();

    if (segment_meta.end_offset > -1
        && (local_segment_end_offset + packet_len) > segment_meta.end_offset as u64)
        || file_size >= max_file_size
    {
        sealup_segment(cache_manager, client_pool, &segment_iden).await?;
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn data_fold_shard_test() {}
}
