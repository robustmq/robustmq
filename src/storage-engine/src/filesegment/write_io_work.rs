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

use super::write_manager::{SegmentWriteResp, WriteChannelData};
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::filesegment::file::open_segment_write;
use crate::filesegment::index::build::{save_index, BuildIndexRaw, IndexTypeEnum};
use crate::filesegment::offset::SegmentOffset;
use crate::filesegment::scroll::{
    is_trigger_next_segment_scroll, trigger_next_segment_scroll, trigger_seal_segment,
    trigger_update_start_timestamp,
};
use crate::filesegment::SegmentIdentity;
use common_base::tools::now_second;
use dashmap::mapref::entry::Entry;
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_read_config::AdapterWriteRespRow;
use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Clone)]
pub struct IoWork {
    offset_data: dashmap::DashMap<String, u64>,
    segment_offset: SegmentOffset,
}

impl IoWork {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        io_seq: u32,
    ) -> Self {
        info!("io worker {} start success", io_seq);
        IoWork {
            offset_data: dashmap::DashMap::with_capacity(16),
            segment_offset: SegmentOffset::new(rocksdb_engine_handler, cache_manager),
        }
    }

    pub fn get_offset(&self, segment_iden: &SegmentIdentity) -> Result<u64, StorageEngineError> {
        let key = segment_iden.name();
        if let Some(offset) = self.offset_data.get(&key) {
            return Ok(*offset);
        }
        // Cache miss: look up the correct next write offset for this segment.
        // get_segment_next_write_offset returns start_offset for brand-new segments
        // (end_offset == 0) so a new leader does not restart writing at 0.
        let result = self
            .segment_offset
            .get_segment_next_write_offset(segment_iden)?;
        self.offset_data.insert(key, result);
        Ok(result)
    }

    pub fn save_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        self.segment_offset
            .save_latest_offset(segment_iden, offset)?;
        self.offset_data.insert(segment_iden.name(), offset);
        Ok(())
    }
}

pub fn create_io_thread(
    io_work: Arc<IoWork>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<StorageCacheManager>,
    client_pool: Arc<ClientPool>,
    mut data_recv: mpsc::Receiver<WriteChannelData>,
    stop_send: tokio::sync::broadcast::Sender<bool>,
) {
    tokio::spawn(Box::pin(async move {
        let mut stop_recv = stop_send.subscribe();

        let mut acc = BatchAccumulator::new();
        let mut tmp_offset_info: HashMap<String, u64> = HashMap::new();

        loop {
            match stop_recv.try_recv() {
                Ok(bl) => {
                    if bl {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
                Err(_) => {}
            }

            let batch = match collect_batch(&mut data_recv).await {
                None => break,
                Some(b) if b.is_empty() => continue,
                Some(b) => b,
            };

            acc.clear();
            tmp_offset_info.clear();

            for channel_data in batch {
                let shard_name = channel_data.segment_iden.shard_name.clone();

                let start_offset = if let Some(&o) = tmp_offset_info.get(&shard_name) {
                    o
                } else {
                    match io_work.get_offset(&channel_data.segment_iden) {
                        Ok(o) => o,
                        Err(ex) => {
                            let segment = channel_data.segment_iden.segment;
                            if let Err(e) = channel_data.resp_sx.send(SegmentWriteResp {
                                error: Some(ex.to_string()),
                                ..Default::default()
                            }) {
                                error!(
                                    "Failed to send get_offset error for shard {}, segment {}: {:?}",
                                    shard_name, segment, e
                                );
                            }
                            continue;
                        }
                    }
                };

                let next_offset =
                    group_channel_data(channel_data, start_offset, &cache_manager, &mut acc);
                tmp_offset_info.insert(shard_name, next_offset);
            }

            for (segment_iden, shard_data) in acc.write_data.iter() {
                let pkid_offset_list = acc.pkid_offset.get(segment_iden).unwrap();
                let index_list = acc
                    .index_list
                    .get(segment_iden)
                    .cloned()
                    .unwrap_or_default();

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
                    Ok(maybe_resp) => {
                        let has_written = maybe_resp.is_some();
                        let mut resp = maybe_resp.unwrap_or_default();
                        if let Some(overflow_list) = acc.overflow_pkids.get(segment_iden) {
                            for pkid in overflow_list {
                                resp.offsets.push(AdapterWriteRespRow {
                                    pkid: *pkid,
                                    need_next_segment: true,
                                    ..Default::default()
                                });
                            }
                        }
                        let ok = if has_written {
                            success_save_offset(
                                &mut acc.sender_list,
                                pkid_offset_list,
                                &io_work,
                                segment_iden,
                            )
                        } else {
                            true
                        };
                        if ok {
                            call_success_response(&mut acc.sender_list, segment_iden, &resp);
                        }
                    }
                    Err(ex) => {
                        call_error_response(&mut acc.sender_list, segment_iden, &ex.to_string());
                    }
                }
            }
        }
    }));
}

/// Drains the channel into a batch of up to 100 items.
/// Returns None when the channel is closed (caller should exit).
/// Returns Some([]) on timeout (caller should continue).
async fn collect_batch(
    data_recv: &mut mpsc::Receiver<WriteChannelData>,
) -> Option<Vec<WriteChannelData>> {
    let first = match timeout(Duration::from_millis(10), data_recv.recv()).await {
        Ok(Some(data)) => data,
        Ok(None) => return None,
        Err(_) => return Some(vec![]),
    };

    let mut batch = vec![first];
    while batch.len() < 100 {
        match data_recv.try_recv() {
            Ok(data) => batch.push(data),
            Err(TryRecvError::Empty) => break,
            Err(e) => {
                error!("Failed to receive write data from channel: {}", e);
                break;
            }
        }
    }
    Some(batch)
}

struct BatchAccumulator {
    write_data: HashMap<SegmentIdentity, Vec<StorageRecord>>,
    pkid_offset: HashMap<SegmentIdentity, HashMap<u64, u64>>,
    sender_list: HashMap<SegmentIdentity, Vec<oneshot::Sender<SegmentWriteResp>>>,
    index_list: HashMap<SegmentIdentity, Vec<BuildIndexRaw>>,
    overflow_pkids: HashMap<SegmentIdentity, Vec<u64>>,
}

impl BatchAccumulator {
    fn new() -> Self {
        BatchAccumulator {
            write_data: HashMap::new(),
            pkid_offset: HashMap::new(),
            sender_list: HashMap::new(),
            index_list: HashMap::new(),
            overflow_pkids: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.write_data.clear();
        self.pkid_offset.clear();
        self.sender_list.clear();
        self.index_list.clear();
        self.overflow_pkids.clear();
    }
}

/// Groups one WriteChannelData into the per-segment maps, applying boundary overflow detection.
/// Returns the next start_offset for this shard after processing all records.
fn group_channel_data(
    channel_data: WriteChannelData,
    start_offset: u64,
    cache_manager: &Arc<StorageCacheManager>,
    acc: &mut BatchAccumulator,
) -> u64 {
    let shard_name = channel_data.segment_iden.shard_name.clone();
    let segment = channel_data.segment_iden.segment;

    let shard_list = acc
        .write_data
        .entry(channel_data.segment_iden.clone())
        .or_default();
    let shard_pkid_list = acc
        .pkid_offset
        .entry(channel_data.segment_iden.clone())
        .or_default();
    let sender_list = acc
        .sender_list
        .entry(channel_data.segment_iden.clone())
        .or_default();
    let index_list = acc
        .index_list
        .entry(channel_data.segment_iden.clone())
        .or_default();
    let overflow_list = acc
        .overflow_pkids
        .entry(channel_data.segment_iden.clone())
        .or_default();

    let seg_end_offset: Option<u64> = cache_manager
        .get_segment_meta(&channel_data.segment_iden)
        .and_then(|m| {
            if m.end_offset > 0 {
                Some(m.end_offset as u64)
            } else {
                None
            }
        });

    sender_list.push(channel_data.resp_sx);

    let create_t = now_second();
    let mut offset = start_offset;

    for row in channel_data.data_list {
        let record_offset = offset;

        if let Some(end) = seg_end_offset {
            if record_offset > end {
                overflow_list.push(row.pkid);
                continue;
            }
        }

        shard_pkid_list.insert(row.pkid, record_offset);
        shard_list.push(StorageRecord {
            metadata: StorageRecordMetadata::new(
                record_offset,
                &shard_name,
                segment,
                &row.header,
                &row.key,
                &row.tags,
                row.expire_at,
                &row.value,
            ),
            data: row.value,
            protocol_data: row.protocol_data,
        });

        if let Some(key) = row.key {
            index_list.push(BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some(key),
                offset: record_offset,
                ..Default::default()
            });
        }

        if let Some(tags) = row.tags {
            for tag in tags {
                index_list.push(BuildIndexRaw {
                    index_type: IndexTypeEnum::Tag,
                    tag: Some(tag),
                    offset: record_offset,
                    ..Default::default()
                });
            }
        }

        if record_offset.is_multiple_of(10000) {
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

        offset += 1;
    }

    offset
}

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

    let offsets: Vec<u64> = data_list.iter().map(|r| r.metadata.offset).collect();
    let last_offset = *offsets.iter().max().unwrap();

    let mut segment_write = match cache_manager.segment_file_writer.entry(segment_iden.name()) {
        Entry::Occupied(e) => e.into_ref(),
        Entry::Vacant(e) => {
            let segment_file = open_segment_write(cache_manager, segment_iden).await?;
            e.insert(segment_file)
        }
    };

    // update start timestamp by segment
    let is_first_write = cache_manager
        .get_segment_meta(segment_iden)
        .map(|meta| offsets.contains(&(meta.start_offset as u64)))
        .unwrap_or(false);

    if is_first_write {
        trigger_update_start_timestamp(
            cache_manager.clone(),
            client_pool.clone(),
            segment_iden.clone(),
        );
    }

    // save data
    let offset_positions = segment_write.write(data_list).await?;

    // save index
    save_index(
        rocksdb_engine_handler,
        segment_iden,
        index_data,
        &offset_positions,
    )?;

    // seal up segment
    let is_end_reached = cache_manager
        .get_segment_meta(segment_iden)
        .map(|meta| meta.end_offset > 0 && offsets.contains(&(meta.end_offset as u64)))
        .unwrap_or(false);

    if is_end_reached {
        let cp = client_pool.clone();
        let si = segment_iden.clone();
        tokio::spawn(async move { trigger_seal_segment(cp, si).await });
    }

    // trigger create next segment
    if is_trigger_next_segment_scroll(&offsets) {
        if let Err(e) = trigger_next_segment_scroll(
            cache_manager,
            client_pool,
            &segment_write,
            segment_iden,
            last_offset,
        )
        .await
        {
            error!("{}", e);
        }
    }

    // collect resp
    let resp_offsets: Vec<AdapterWriteRespRow> = pkid_offset_list
        .iter()
        .map(|(&pkid, &offset)| AdapterWriteRespRow {
            pkid,
            offset,
            ..Default::default()
        })
        .collect();

    Ok(Some(SegmentWriteResp {
        offsets: resp_offsets,
        last_offset,
        ..Default::default()
    }))
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
                    "Failed to send write response for shard {}, segment {}: {:?}",
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
                    "Failed to send error response for shard {}, segment {}: {:?}",
                    segment_iden.shard_name, segment_iden.segment, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_init_segment;
    use crate::filesegment::SegmentIdentity;
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
    use tokio::sync::oneshot;

    use super::super::write_manager::WriteChannelDataRecord;

    fn make_record(pkid: u64, value: &str) -> WriteChannelDataRecord {
        WriteChannelDataRecord {
            pkid,
            header: None,
            key: None,
            tags: None,
            value: Bytes::from(value.to_string()),
            protocol_data: None,
            expire_at: 0,
        }
    }

    #[tokio::test]
    async fn collect_batch_drains_channel_test() {
        let (tx, mut rx) = mpsc::channel::<WriteChannelData>(10);
        let segment_iden = SegmentIdentity {
            shard_name: "s".into(),
            segment: 0,
        };

        for _i in 0..3u64 {
            let (resp_sx, _) = oneshot::channel();
            tx.send(WriteChannelData {
                segment_iden: segment_iden.clone(),
                data_list: vec![],
                resp_sx,
            })
            .await
            .unwrap();
        }
        drop(tx);

        let batch = collect_batch(&mut rx).await.unwrap();
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn group_channel_data_overflow_test() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (_, cache_manager, _, _) = test_init_segment(StorageType::EngineSegment).await;

            let segment_iden = SegmentIdentity {
                shard_name: "test-shard".into(),
                segment: 0,
            };

            // Set end_offset = 9999 in segment meta
            cache_manager.set_segment_meta(EngineSegmentMetadata {
                shard_name: segment_iden.shard_name.clone(),
                segment_seq: segment_iden.segment,
                start_offset: 0,
                end_offset: 9999,
                start_timestamp: 0,
                end_timestamp: 0,
            });

            let (resp_sx, _) = oneshot::channel();
            let channel_data = WriteChannelData {
                segment_iden: segment_iden.clone(),
                data_list: vec![
                    make_record(1, "a"), // offset 9998 → normal
                    make_record(2, "b"), // offset 9999 → normal (= end_offset)
                    make_record(3, "c"), // offset 10000 → overflow
                    make_record(4, "d"), // offset 10001 → overflow
                ],
                resp_sx,
            };

            let mut acc = BatchAccumulator::new();

            let next = group_channel_data(channel_data, 9998, &cache_manager, &mut acc);

            // offsets 9998 and 9999 were written
            let written = acc.write_data.get(&segment_iden).unwrap();
            assert_eq!(written.len(), 2);

            // pkids 3 and 4 overflowed
            let overflow = acc.overflow_pkids.get(&segment_iden).unwrap();
            assert_eq!(overflow.len(), 2);
            assert!(overflow.contains(&3));
            assert!(overflow.contains(&4));

            // counter advanced only for written records
            assert_eq!(next, 10000);
        });
    }

    #[tokio::test]
    async fn batch_write_test() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let client_pool = Arc::new(ClientPool::new(100));

        let records: Vec<StorageRecord> = (0..5u64)
            .map(|i| {
                let data = Bytes::from(format!("data-{}", i));
                StorageRecord {
                    metadata: StorageRecordMetadata::new(
                        i,
                        &segment_iden.shard_name,
                        segment_iden.segment,
                        &None,
                        &None,
                        &None,
                        0,
                        &data,
                    ),
                    data,
                    protocol_data: None,
                }
            })
            .collect();

        let pkid_offset: HashMap<u64, u64> = (0..5u64).map(|i| (i, i)).collect();

        let resp = batch_write(
            &cache_manager,
            &rocksdb,
            &client_pool,
            &segment_iden,
            &records,
            &pkid_offset,
            &[],
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(resp.offsets.len(), 5);
        assert_eq!(resp.last_offset, 4);
    }
}
