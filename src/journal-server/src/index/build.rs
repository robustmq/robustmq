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

use common_base::tools::now_second;
use log::{debug, error, warn};
use metadata_struct::journal::segment::SegmentStatus;
use rocksdb_engine::engine::{
    rocksdb_engine_delete, rocksdb_engine_exists, rocksdb_engine_get, rocksdb_engine_prefix_map,
    rocksdb_engine_save,
};
use rocksdb_engine::RocksDBEngine;
use tokio::select;
use tokio::sync::broadcast::{self, Receiver};
use tokio::time::sleep;

use super::keys::{finish_build_index, last_offset_build_index, segment_index_prefix};
use super::offset::OffsetIndexManager;
use super::tag::TagIndexManager;
use super::time::TimestampIndexManager;
use crate::core::cache::CacheManager;
use crate::core::consts::{BUILD_INDE_PER_RECORD_NUM, DB_COLUMN_FAMILY_INDEX};
use crate::core::error::JournalServerError;
use crate::index::IndexData;
use crate::segment::file::{open_segment_write, ReadData};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct IndexBuildThreadData {
    pub last_build_time: u64,
    pub stop_send: broadcast::Sender<bool>,
}

pub async fn try_trigger_build_index(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    if cache_manager.contain_build_index_thread(segment_iden) {
        return Ok(());
    }

    if is_finish_build_index(rocksdb_engine_handler, segment_iden)? {
        warn!("segment {} is in the wrong state, marking that the index has been built but data is still being written.",
        segment_iden.name());
        return Ok(());
    }

    // Get the end offset of the local segment file
    let segment_file_meta =
        if let Some(segment_file) = segment_file_manager.get_segment_file(segment_iden) {
            segment_file
        } else {
            return Err(JournalServerError::SegmentMetaNotExists(
                segment_iden.name(),
            ));
        };

    // get start read position
    let last_build_offset =
        (get_last_offset_build_index(rocksdb_engine_handler, segment_iden)?).unwrap_or(0);
    let offset_index = OffsetIndexManager::new(rocksdb_engine_handler.clone());
    let start_position = if let Some(position) = offset_index
        .get_last_nearest_position_by_offset(segment_iden, last_build_offset)
        .await?
    {
        position.position
    } else {
        0
    };

    let (stop_sender, stop_recv) = broadcast::channel::<bool>(1);

    start_segment_build_index_thread(
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
        segment_iden.clone(),
        segment_file_meta.start_offset as u64,
        start_position,
        last_build_offset,
        stop_recv,
    )
    .await?;

    let index_thread_data = IndexBuildThreadData {
        stop_send: stop_sender.clone(),
        last_build_time: now_second(),
    };
    cache_manager.add_build_index_thread(segment_iden, index_thread_data);
    Ok(())
}

async fn start_segment_build_index_thread(
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_iden: SegmentIdentity,
    start_offset: u64,
    start_position: u64,
    mut last_build_offset: u64,
    mut stop_recv: Receiver<bool>,
) -> Result<(), JournalServerError> {
    let offset_index = OffsetIndexManager::new(rocksdb_engine_handler.clone());
    let time_index = TimestampIndexManager::new(rocksdb_engine_handler.clone());
    let tag_index = TagIndexManager::new(rocksdb_engine_handler.clone());
    let (segment_write, _) = open_segment_write(&cache_manager, &segment_iden).await?;

    tokio::spawn(async move {
        let size = 10 * 1024 * 1024;
        let mut data_empty_times = 0;
        let max_data_empty_times = 10 * 60;

        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            cache_manager.remove_build_index_thread(&segment_iden);
                            debug!("segment {} index build thread exited successfully.",
                               segment_iden.name());
                            break;
                        }
                    }
                },
                val = segment_write.read_by_offset(start_position, last_build_offset, size)=>{
                    match val {
                        Ok(data) => {
                            if data.is_empty() {

                                if data_empty_times >= max_data_empty_times {
                                    try_finish_segment_index_build(&rocksdb_engine_handler, &cache_manager, &segment_iden);
                                    cache_manager.remove_build_index_thread(&segment_iden);

                                    debug!("segment {} No data after 10 minutes of noise, index building thread temporarily quit",
                                        segment_iden.name());
                                    break;
                                }

                                data_empty_times += 1;
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            }

                            data_empty_times = 0;

                            // save offset data
                            if let Err(e) = save_record_index(
                                &data,
                                start_offset,
                                &segment_iden,
                                &offset_index,
                                &time_index,
                                &tag_index)
                            {
                                error!("{}",e);
                                continue;
                            }

                            // save last offset bye build index
                            if let Err(e) = save_last_offset_build_index(
                                &rocksdb_engine_handler,
                                &segment_iden,
                            ) {
                                error!("Failure to save last_offset_build_index information with error message :{}",e);
                                continue;
                            }

                            last_build_offset = data.last().unwrap().record.offset as u64;

                        }
                        Err(e) => {
                            error!("Failed to read Segment file data with error message :{}", e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

fn try_finish_segment_index_build(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    cache_manager: &Arc<CacheManager>,
    segment_iden: &SegmentIdentity,
) {
    if let Some(segment) = cache_manager.get_segment(segment_iden) {
        if segment.status == SegmentStatus::SealUp {
            if let Err(e) = save_finish_build_index(rocksdb_engine_handler, segment_iden) {
                error!("{}", e);
            }
        }
    }
}

fn save_record_index(
    data: &[ReadData],
    start_offset: u64,
    segment_iden: &SegmentIdentity,
    offset_index: &OffsetIndexManager,
    time_index: &TimestampIndexManager,
    tag_index: &TagIndexManager,
) -> Result<(), JournalServerError> {
    for read_data in data.iter() {
        let record = read_data.record.clone();

        let index_data = IndexData {
            offset: record.offset as u64,
            timestamp: record.create_time,
            position: read_data.position,
        };

        if (record.offset - start_offset as i64) % BUILD_INDE_PER_RECORD_NUM as i64 == 0 {
            // build position index
            offset_index.save_position_offset(
                segment_iden,
                record.offset as u64,
                index_data.clone(),
            )?;

            // build timestamp index
            time_index.save_timestamp_offset(
                segment_iden,
                record.create_time,
                index_data.clone(),
            )?;
        }

        // build key index
        if !record.key.is_empty() {
            tag_index.save_key_position(segment_iden, record.key, index_data.clone())?;
        }

        // build tag index
        for tag in record.tags {
            tag_index.save_tag_position(segment_iden, tag, index_data.clone())?;
        }
    }
    Ok(())
}

fn save_finish_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    let key = finish_build_index(segment_iden);
    Ok(rocksdb_engine_save(
        rocksdb_engine_handler.clone(),
        DB_COLUMN_FAMILY_INDEX,
        key,
        true,
    )?)
}

fn is_finish_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<bool, JournalServerError> {
    let key = finish_build_index(segment_iden);
    Ok(rocksdb_engine_exists(
        rocksdb_engine_handler.clone(),
        DB_COLUMN_FAMILY_INDEX,
        key,
    )?)
}

fn remove_last_offset_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    let key = last_offset_build_index(segment_iden);
    Ok(rocksdb_engine_delete(
        rocksdb_engine_handler.clone(),
        DB_COLUMN_FAMILY_INDEX,
        key,
    )?)
}

fn save_last_offset_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    let key = last_offset_build_index(segment_iden);
    Ok(rocksdb_engine_save(
        rocksdb_engine_handler.clone(),
        DB_COLUMN_FAMILY_INDEX,
        key,
        now_second(),
    )?)
}

fn get_last_offset_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<Option<u64>, JournalServerError> {
    let key = last_offset_build_index(segment_iden);
    if let Some(res) =
        rocksdb_engine_get(rocksdb_engine_handler.clone(), DB_COLUMN_FAMILY_INDEX, key)?
    {
        return Ok(Some(serde_json::from_slice::<u64>(&res.data)?));
    }

    Ok(None)
}

pub fn delete_segment_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    let prefix_key_name = segment_index_prefix(segment_iden);
    let comlumn_family = DB_COLUMN_FAMILY_INDEX;
    let data = rocksdb_engine_prefix_map(
        rocksdb_engine_handler.clone(),
        comlumn_family,
        prefix_key_name,
    )?;
    for raw in data.iter() {
        rocksdb_engine_delete(
            rocksdb_engine_handler.clone(),
            comlumn_family,
            raw.key().to_string(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::tools::now_second;
    use metadata_struct::journal::segment::{JournalSegment, Replica};
    use rocksdb_engine::engine::rocksdb_engine_prefix_map;

    use super::{save_finish_build_index, save_last_offset_build_index, try_trigger_build_index};
    use crate::core::cache::CacheManager;
    use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
    use crate::core::test::{test_build_data_fold, test_build_rocksdb_sgement, test_init_conf};
    use crate::index::build::{
        delete_segment_index, get_last_offset_build_index, is_finish_build_index,
        remove_last_offset_build_index,
    };
    use crate::index::keys::segment_index_prefix;
    use crate::index::offset::OffsetIndexManager;
    use crate::index::IndexData;
    use crate::segment::file::SegmentFile;
    use crate::segment::manager::{SegmentFileManager, SegmentFileMetadata};
    #[test]
    fn last_offset_build_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();
        let res = save_last_offset_build_index(&rocksdb_engine_handler, &segment_iden);
        assert!(res.is_ok());

        let res = get_last_offset_build_index(&rocksdb_engine_handler, &segment_iden);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert!(res.unwrap().is_some());

        let res = remove_last_offset_build_index(&rocksdb_engine_handler, &segment_iden);

        let res = get_last_offset_build_index(&rocksdb_engine_handler, &segment_iden);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[test]
    fn finish_build_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let res = save_finish_build_index(&rocksdb_engine_handler, &segment_iden);
        assert!(res.is_ok());

        let res = is_finish_build_index(&rocksdb_engine_handler, &segment_iden);
        assert!(res.is_ok());
        assert!(res.unwrap());
    }

    #[test]
    fn delete_segment_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        // build index
        let offset_index = OffsetIndexManager::new(rocksdb_engine_handler.clone());
        let timestamp = now_second();
        for i in 0..10 {
            let cur_timestamp = timestamp + i * 10;
            let index_data = IndexData {
                offset: i,
                timestamp: cur_timestamp,
                position: i * 5,
            };
            let res = offset_index.save_position_offset(&segment_iden, i, index_data);
            assert!(res.is_ok());
        }

        // check data
        let prefix_key_name = segment_index_prefix(&segment_iden);
        let comlumn_family = DB_COLUMN_FAMILY_INDEX;
        let data = rocksdb_engine_prefix_map(
            rocksdb_engine_handler.clone(),
            comlumn_family,
            prefix_key_name,
        )
        .unwrap();
        assert!(!data.is_empty());

        // delete segment_index
        let res = delete_segment_index(&rocksdb_engine_handler, &segment_iden);
        assert!(res.is_ok());

        // check data
        let prefix_key_name = segment_index_prefix(&segment_iden);
        let comlumn_family = DB_COLUMN_FAMILY_INDEX;
        let data = rocksdb_engine_prefix_map(
            rocksdb_engine_handler.clone(),
            comlumn_family,
            prefix_key_name,
        )
        .unwrap();

        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn build_thread_test() {
        test_init_conf();

        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();
        let data_fold = test_build_data_fold().first().unwrap().to_string();

        let cache_manager = Arc::new(CacheManager::new());
        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        let segment_file = SegmentFileMetadata {
            namespace: segment_iden.namespace.to_string(),
            shard_name: segment_iden.shard_name.to_string(),
            segment_no: segment_iden.segment_seq,
            ..Default::default()
        };
        segment_file_manager.add_segment_file(segment_file);

        // create segment file
        let segment_file = SegmentFile::new(
            segment_iden.namespace.clone(),
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            data_fold.clone(),
        );
        let res = segment_file.try_create().await;
        assert!(res.is_ok());

        // add cache
        let segment = JournalSegment {
            namespace: segment_iden.namespace.clone(),
            shard_name: segment_iden.shard_name.clone(),
            segment_seq: segment_iden.segment_seq,
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: data_fold.clone(),
            }],
            ..Default::default()
        };
        cache_manager.set_segment(segment);

        // start build thread
        let res = try_trigger_build_index(
            &cache_manager,
            &segment_file_manager,
            &rocksdb_engine_handler,
            &segment_iden,
        )
        .await;
        println!("{:?}", res);
        assert!(res.is_ok());

        // check
        assert!(segment_file.exists());
        assert!(cache_manager.get_segment(&segment_iden).is_some());
        assert!(segment_file_manager
            .get_segment_file(&segment_iden)
            .is_some());

        // write data

        // check index
    }
}
