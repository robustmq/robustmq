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
use metadata_struct::journal::segment::SegmentStatus;
use rocksdb_engine::engine::{
    rocksdb_engine_delete, rocksdb_engine_get, rocksdb_engine_list_by_prefix_to_map,
    rocksdb_engine_save,
};
use rocksdb_engine::RocksDBEngine;
use tokio::select;
use tokio::sync::broadcast::{self, Receiver};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

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
                            info!("segment {} index build thread exited successfully.",
                               segment_iden.name());
                            break;
                        }
                    }
                },
                val = segment_write.read_by_offset(start_position, last_build_offset, size, 100)=>{
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
                                &tag_index,
                            ).await
                            {
                                error!("{}",e);
                                continue;
                            }

                            last_build_offset = data.last().unwrap().record.offset as u64;
                            // save last offset bye build index
                            if let Err(e) = save_last_offset_build_index(
                                &rocksdb_engine_handler,
                                &segment_iden,
                                last_build_offset,
                            ) {
                                error!("Failure to save last_offset_build_index information with error message :{}",e);
                                continue;
                            }


                        }
                        Err(e) => {
                            error!("Failed to read Segment file data with error message:{},segment:{:?}", e, segment_iden);

                            if e.to_string().contains("No such file or directory"){
                                cache_manager.remove_build_index_thread(&segment_iden);
                                break;
                            }

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

async fn save_record_index(
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

        if read_data.position == 0 {
            offset_index.save_start_offset(segment_iden, record.offset as u64)?;
        }

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
        now_second(),
    )?)
}

fn is_finish_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<bool, JournalServerError> {
    let key = finish_build_index(segment_iden);
    let res = rocksdb_engine_get(rocksdb_engine_handler.clone(), DB_COLUMN_FAMILY_INDEX, key)?;
    Ok(res.is_some())
}

fn save_last_offset_build_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    offset: u64,
) -> Result<(), JournalServerError> {
    let key = last_offset_build_index(segment_iden);
    Ok(rocksdb_engine_save(
        rocksdb_engine_handler.clone(),
        DB_COLUMN_FAMILY_INDEX,
        key,
        offset,
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
        return Ok(Some(serde_json::from_str::<u64>(&res.data)?));
    }

    Ok(None)
}

pub fn delete_segment_index(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    let prefix_key_name = segment_index_prefix(segment_iden);
    let comlumn_family = DB_COLUMN_FAMILY_INDEX;
    let data = rocksdb_engine_list_by_prefix_to_map(
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

    use std::time::Duration;

    use common_base::tools::now_second;
    use rocksdb_engine::engine::rocksdb_engine_list_by_prefix_to_map;
    use tokio::time::sleep;

    use super::{save_finish_build_index, save_last_offset_build_index, try_trigger_build_index};
    use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
    use crate::core::test::{test_base_write_data, test_build_rocksdb_sgement};
    use crate::index::build::{
        delete_segment_index, get_last_offset_build_index, is_finish_build_index,
    };
    use crate::index::keys::segment_index_prefix;
    use crate::index::offset::OffsetIndexManager;
    use crate::index::IndexData;
    #[test]
    fn last_offset_build_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();
        let res = save_last_offset_build_index(&rocksdb_engine_handler, &segment_iden, 10);
        assert!(res.is_ok());

        let res = get_last_offset_build_index(&rocksdb_engine_handler, &segment_iden);
        println!("{res:?}");
        assert!(res.is_ok());
        assert!(res.unwrap().is_some());
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
        let data = rocksdb_engine_list_by_prefix_to_map(
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
        let data = rocksdb_engine_list_by_prefix_to_map(
            rocksdb_engine_handler.clone(),
            comlumn_family,
            prefix_key_name,
        )
        .unwrap();

        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn build_thread_test() {
        let (segment_iden, cache_manager, segment_file_manager, _, rocksdb_engine_handler) =
            test_base_write_data(10001).await;
        let res: Result<(), crate::core::error::JournalServerError> = try_trigger_build_index(
            &cache_manager,
            &segment_file_manager,
            &rocksdb_engine_handler,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(120)).await;
        let prefix_key_name = segment_index_prefix(&segment_iden);
        let comlumn_family = DB_COLUMN_FAMILY_INDEX;
        let data = rocksdb_engine_list_by_prefix_to_map(
            rocksdb_engine_handler.clone(),
            comlumn_family,
            prefix_key_name,
        )
        .unwrap();

        let mut tag_num = 0;
        let mut key_num = 0;
        let mut offset_num = 0;
        let mut timestamp_num = 0;

        for (key, val) in data {
            if key.contains("tag") {
                tag_num += 1;
            }

            if key.contains("key") {
                key_num += 1;
            }

            if key.contains("last/offset") {
                let last_offset = serde_json::from_str::<i64>(&val.data).unwrap();
                assert_eq!(last_offset, 10000);
            }

            if key.contains("offset/position") {
                let last_offset = serde_json::from_str::<IndexData>(&val.data).unwrap();
                println!("key: {key},val={last_offset:?}");
                assert_eq!(last_offset.offset, 9999);
                offset_num += 1;
            }

            if key.contains("timestamp/time-") {
                let last_offset = serde_json::from_str::<IndexData>(&val.data).unwrap();
                println!("key: {key},val={last_offset:?}");
                assert_eq!(last_offset.offset, 9999);
                timestamp_num += 1;
            }
        }

        assert_eq!(tag_num, 10001);
        assert_eq!(key_num, 10001);
        assert_eq!(offset_num, 1);
        assert_eq!(timestamp_num, 1);
    }
}
