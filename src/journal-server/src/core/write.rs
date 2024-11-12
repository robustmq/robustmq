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
use log::{debug, error};
use metadata_struct::journal::segment::SegmentStatus;
use protocol::journal_server::journal_engine::{
    WriteReqBody, WriteRespMessage, WriteRespMessageStatus,
};
use protocol::journal_server::journal_record::JournalRecord;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;

use super::cache::CacheManager;
use super::error::{get_journal_server_code, JournalServerError};
use crate::segment::file::SegmentFile;
use crate::segment::manager::SegmentFileManager;

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
    segment_write: DashMap<String, SegmentWrite>,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
}

impl WriteManager {
    pub fn new(
        segment_file_manager: Arc<SegmentFileManager>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        WriteManager {
            segment_write: DashMap::with_capacity(8),
            segment_file_manager,
            cache_manager,
        }
    }

    pub async fn write(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        datas: Vec<JournalRecord>,
    ) -> Result<SegmentWriteResp, JournalServerError> {
        let write = self.get_write(namespace, shard_name, segment).await?;
        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();

        let data = SegmentWriteData {
            data: datas,
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

            start_segment_sync_write_thread(
                namespace,
                shard_name,
                segment,
                self.segment_file_manager.clone(),
                self.cache_manager.clone(),
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
    namespace: &str,
    shard_name: &str,
    segment_no: u32,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
    mut data_recv: Receiver<SegmentWriteData>,
    mut stop_recv: broadcast::Receiver<bool>,
) -> Result<(), JournalServerError> {
    let segment_file_meta = if let Some(segment_file) =
        segment_file_manager.get_segment_file(namespace, shard_name, segment_no)
    {
        segment_file
    } else {
        return Err(JournalServerError::SegmentFileNotExists(format!(
            "{}-{}",
            shard_name, segment_no
        )));
    };

    let mut end_offset = segment_file_meta.end_offset;

    let raw_namespace = namespace.to_string();
    let raw_shard_name = shard_name.to_string();
    let raw_cache_manager = cache_manager.clone();

    let file_size_50_percent = false;
    let file_size_98_percent = false;

    let (segment_write, max_file_size) = open_segment_write(
        namespace,
        shard_name,
        segment_no,
        segment_file_manager.clone(),
        cache_manager,
    )
    .await?;

    tokio::spawn(async move {
        debug!("");

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

                        // check if the offset exceeds the end offset
                        let segment_meta =if let Some(segment) = raw_cache_manager.get_segment_meta(&raw_namespace, &raw_shard_name, segment_no) {
                            segment
                        } else {
                            continue;
                        };

                        if segment_meta.end_offset > 0 &&  (end_offset + packet.data.len() as u64) > segment_meta.end_offset as u64{
                                raw_cache_manager.update_segment_status(&raw_namespace, &raw_shard_name, segment_no,SegmentStatus::PreSealUp);
                                continue;
                        }

                        // check file size
                        let file_size = (segment_write.size().await).unwrap_or_default();

                        if file_size >= max_file_size{
                            raw_cache_manager.update_segment_status(&raw_namespace, &raw_shard_name, segment_no,SegmentStatus::PreSealUp);
                            continue;
                        }

                        // build write data
                        let mut offsets = Vec::new();
                        let mut records =  Vec::new();
                        for mut record in packet.data{

                            let offset = end_offset+1;

                            record.offset = offset;
                            end_offset = offset;

                            offsets.push(offset);
                            records.push(record);
                        }

                        // batch write data
                        let mut resp = SegmentWriteResp::default();
                        match segment_write.write(&records).await {
                            Ok(()) => {
                                resp.offset = offsets.clone();
                            }
                            Err(e) => {
                                resp.error = Some(e);
                            }
                        }

                        // update segment end offset
                        if let Some(end_offset) = offsets.last(){
                            match segment_file_manager.update_end_offset(&raw_namespace,&raw_shard_name,segment_no,*end_offset){
                                Ok(()) =>{}
                                Err(e) => {
                                    error!("{}",e);
                                }
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

async fn open_segment_write(
    namespace: &str,
    shard_name: &str,
    segment_no: u32,
    segment_file_manager: Arc<SegmentFileManager>,
    cache_manager: Arc<CacheManager>,
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

        let mut datas = Vec::new();
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
            datas.push(record);
        }

        let resp = write_manager
            .write(
                &shard_data.namespace,
                &shard_data.shard_name,
                shard_data.segment,
                datas,
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
                    // offset: resp_raw.offset,
                    ..Default::default()
                }
            };
            resp_message_status.push(status);
        }

        resp_message.messages = resp_message_status;
        results.push(resp_message);
    }
    Ok(Vec::new())
}
