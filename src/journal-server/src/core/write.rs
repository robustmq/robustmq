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

use std::sync::Arc;
use std::time::Duration;

use common_base::config::journal_server::journal_server_conf;
use common_base::tools::now_second;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use log::{debug, error};
use metadata_struct::journal::segment::segment_name;
use protocol::journal_server::journal_engine::{
    WriteReqBody, WriteRespMessage, WriteRespMessageStatus,
};
use protocol::journal_server::journal_record::JournalRecord;
use rocksdb_engine::RocksDBEngine;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;

use super::cache::CacheManager;
use super::error::{get_journal_server_code, JournalServerError};
use crate::index::build::try_trigger_build_index;
use crate::segment::file::SegmentFile;
use crate::segment::manager::SegmentFileManager;
use crate::segment::status::segment_status_to_sealup;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct SegmentWrite {
    last_write_time: u64,
    data_sender: Sender<SegmentWriteData>,
    stop_sender: broadcast::Sender<bool>,
}

pub struct SegmentWriteData {
    data: Vec<JournalRecord>,
    resp_sx: oneshot::Sender<SegmentWriteResp>,
}

#[derive(Default)]
pub struct SegmentWriteResp {
    offset: Vec<u64>,
    error: Option<JournalServerError>,
}

pub struct WriteManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_write: DashMap<String, SegmentWrite>,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
}

impl WriteManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        segment_file_manager: Arc<SegmentFileManager>,
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        WriteManager {
            rocksdb_engine_handler,
            segment_write: DashMap::with_capacity(8),
            segment_file_manager,
            cache_manager,
            client_pool,
        }
    }

    pub async fn write(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        data_list: Vec<JournalRecord>,
    ) -> Result<SegmentWriteResp, JournalServerError> {
        let write = self.get_write(namespace, shard_name, segment).await?;
        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();

        let data = SegmentWriteData {
            data: data_list,
            resp_sx: sx,
        };
        write.data_sender.send(data).await?;

        let time_res = timeout(Duration::from_secs(30), rx).await?;

        Ok(time_res?)
    }

    pub fn stop_write(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) -> Result<(), JournalServerError> {
        let key = self.key(namespace, shard_name, segment);
        if let Some(write) = self.segment_write.get(&key) {
            write.stop_sender.send(true)?;
            self.segment_write.remove(&key);
        }
        Ok(())
    }

    async fn get_write(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) -> Result<SegmentWrite, JournalServerError> {
        let key = self.key(namespace, shard_name, segment);
        let write = if let Some(write) = self.segment_write.get(&key) {
            write.clone()
        } else {
            let (sender, recv) = mpsc::channel::<SegmentWriteData>(1000);
            let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

            let segment_iden = SegmentIdentity {
                namespace: namespace.to_string(),
                shard_name: shard_name.to_string(),
                segment_seq: segment,
            };
            start_segment_sync_write_thread(
                self.rocksdb_engine_handler.clone(),
                segment_iden,
                self.segment_file_manager.clone(),
                self.cache_manager.clone(),
                self.client_pool.clone(),
                recv,
                stop_recv,
            )
            .await?;

            let write = SegmentWrite {
                last_write_time: 0,
                data_sender: sender,
                stop_sender,
            };
            self.segment_write.insert(key, write.clone());
            write
        };
        Ok(write)
    }

    fn key(&self, namespace: &str, shard_name: &str, segment: u32) -> String {
        format!("{}_{}_{}", namespace, shard_name, segment)
    }
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
    let namespace = &segment_iden.namespace;
    let shard_name = &segment_iden.shard_name;
    let segment_no = segment_iden.segment_seq;

    // Get the end offset of the local segment file
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

    // build tokio task param
    let raw_namespace = namespace.to_string();
    let raw_shard_name = shard_name.to_string();
    let raw_cache_manager = cache_manager.clone();
    let (segment_write, max_file_size) =
        open_segment_write(cache_manager.clone(), namespace, shard_name, segment_no).await?;

    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("{}","TCP Server handler thread stopped successfully.");
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
                        match is_sealup_segment(
                            &raw_cache_manager,
                            &client_pool,
                            &segment_write,
                            local_segment_end_offset,
                            packet.data.len() as u64,
                            max_file_size,
                        ).await{
                            Ok(sealup) => {
                                if sealup{
                                    resp.error = Some(JournalServerError::SegmentAlreadySealUp(segment_name(&raw_namespace,&raw_shard_name,segment_no)));
                                }else{
                                    match batch_write_segment(
                                        &packet,
                                        &segment_write,
                                        &segment_file_manager,
                                        local_segment_end_offset).await
                                    {
                                        Ok(resp_data) =>{
                                            if let Some(end_offset) = resp_data.offset.last() {
                                                local_segment_end_offset = *end_offset;
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
    mut local_segment_end_offset: u64,
) -> Result<SegmentWriteResp, JournalServerError> {
    let namespace = &segment_write.namespace;
    let shard_name = &segment_write.shard_name;
    let segment_no = segment_write.segment_no;

    let mut resp = SegmentWriteResp::default();

    // build write data
    let mut offsets = Vec::new();
    let mut records = Vec::new();
    for mut record in packet.data.clone() {
        record.offset = local_segment_end_offset + 1;
        local_segment_end_offset = record.offset;
        offsets.push(record.offset);
        records.push(record);
    }

    // batch write data
    match segment_write.write(&records).await {
        Ok(()) => {
            resp.offset = offsets.clone();
        }
        Err(e) => {
            resp.error = Some(e);
        }
    }

    // update local segment file end offset
    if let Some(end_offset) = offsets.last() {
        segment_file_manager.update_end_offset(namespace, shard_name, segment_no, *end_offset)?;
    }

    Ok(resp)
}

async fn is_sealup_segment(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    segment_write: &SegmentFile,
    local_segment_end_offset: u64,
    packet_len: u64,
    max_file_size: u64,
) -> Result<bool, JournalServerError> {
    let namespace = &segment_write.namespace;
    let shard_name = &segment_write.shard_name;
    let segment_no = segment_write.segment_no;

    // check if the offset exceeds the end offset
    let segment_meta =
        if let Some(segment) = cache_manager.get_segment_meta(namespace, shard_name, segment_no) {
            segment
        } else {
            return Err(JournalServerError::SegmentMetaNotExists(format!(
                "{},{},{}",
                namespace, shard_name, segment_no
            )));
        };

    if segment_meta.end_offset > -1
        && (local_segment_end_offset + packet_len) > segment_meta.end_offset as u64
    {
        segment_status_to_sealup(
            cache_manager,
            client_pool,
            namespace,
            shard_name,
            segment_no,
        )
        .await?;
        return Ok(true);
    }

    // check file size
    let file_size = (segment_write.size().await).unwrap_or_default();
    if file_size >= max_file_size {
        segment_status_to_sealup(
            cache_manager,
            client_pool,
            namespace,
            shard_name,
            segment_no,
        )
        .await?;
        return Ok(true);
    }

    Ok(false)
}

pub async fn open_segment_write(
    cache_manager: Arc<CacheManager>,
    namespace: &str,
    shard_name: &str,
    segment_no: u32,
) -> Result<(SegmentFile, u64), JournalServerError> {
    let segment =
        if let Some(segment) = cache_manager.get_segment(namespace, shard_name, segment_no) {
            segment
        } else {
            return Err(JournalServerError::SegmentNotExist(
                shard_name.to_string(),
                segment_no,
            ));
        };

    let conf = journal_server_conf();
    let fold = if let Some(fold) = segment.get_fold(conf.node_id) {
        fold
    } else {
        return Err(JournalServerError::SegmentDataDirectoryNotFound(
            format!("{}-{}", shard_name, segment_no),
            conf.node_id,
        ));
    };

    Ok((
        SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            fold,
        ),
        segment.config.max_segment_size,
    ))
}

pub async fn write_data(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    write_manager: &Arc<WriteManager>,
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

        let mut data_list = Vec::new();
        for message in shard_data.messages {
            // todo data validator
            let record = JournalRecord {
                content: message.value,
                create_time: now_second(),
                key: message.key,
                namespace: shard_data.namespace.clone(),
                shard_name: shard_data.shard_name.clone(),
                segment: shard_data.segment,
                tags: message.tags,
                ..Default::default()
            };
            data_list.push(record);
        }

        let resp = write_manager
            .write(
                &shard_data.namespace,
                &shard_data.shard_name,
                shard_data.segment,
                data_list,
            )
            .await?;

        let mut resp_message_status = Vec::new();

        let is_error = resp.error.is_some();
        let error = if is_error {
            let err = resp.error.unwrap();
            Some(
                protocol::journal_server::journal_engine::JournalEngineError {
                    code: get_journal_server_code(&err),
                    error: err.to_string(),
                },
            )
        } else {
            None
        };

        for resp_raw in resp.offset {
            let status = if is_error {
                WriteRespMessageStatus {
                    error: error.clone(),
                    ..Default::default()
                }
            } else {
                WriteRespMessageStatus {
                    offset: resp_raw,
                    ..Default::default()
                }
            };
            resp_message_status.push(status);
        }

        resp_message.messages = resp_message_status;
        results.push(resp_message);
    }
    Ok(results)
}
