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

use dashmap::DashMap;
use log::{error, info};
use metadata_struct::journal::segment::segment_name;
use rocksdb_engine::engine::{
    rocksdb_engine_delete, rocksdb_engine_exists, rocksdb_engine_get, rocksdb_engine_prefix_map,
    rocksdb_engine_save,
};
use rocksdb_engine::RocksDBEngine;
use tokio::sync::broadcast;
use tokio::time::sleep;

use super::engine::DB_COLUMN_FAMILY_INDEX;
use super::keys::{finish_build_index, last_offset_build_index, segment_index_prefix};
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::write::open_segment_write;
use crate::segment::SegmentIdentity;

pub struct IndexBuildManager {
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    index_segments: DashMap<String, SegmentIdentity>,
    build_index_thread: DashMap<String, broadcast::Sender<bool>>,
}

impl IndexBuildManager {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        let index_segments = DashMap::with_capacity(2);
        let build_index_thread = DashMap::with_capacity(2);
        IndexBuildManager {
            cache_manager,
            rocksdb_engine_handler,
            index_segments,
            build_index_thread,
        }
    }

    pub fn add_index_segment(
        &self,
        segment_iden: SegmentIdentity,
    ) -> Result<(), JournalServerError> {
        let key = segment_name(
            &segment_iden.namespace,
            &segment_iden.shard_name,
            segment_iden.segment_seq,
        );
        self.index_segments.insert(key, segment_iden.clone());
        if !is_finish_build_index(
            self.rocksdb_engine_handler.clone(),
            &segment_iden.namespace,
            &segment_iden.shard_name,
            segment_iden.segment_seq,
        )? {
            self.start_segment_build_index_thread(
                self.cache_manager.clone(),
                &segment_iden.namespace,
                &segment_iden.shard_name,
                segment_iden.segment_seq,
            );
        }
        Ok(())
    }

    pub fn try_trigger_build_index(&self, namespace: &str, shard_name: &str, segment: u32) {
        let key = segment_name(namespace, shard_name, segment);
        if !self.build_index_thread.contains_key(&key) {
            self.start_segment_build_index_thread(
                self.cache_manager.clone(),
                namespace,
                shard_name,
                segment,
            );
        }
    }

    pub fn remode_index_segment(&self, namespace: &str, shard_name: &str, segment: u32) {
        let key = segment_name(namespace, shard_name, segment);
        self.index_segments.remove(&key);
    }

    pub fn start_segment_build_index_thread(
        &self,
        cache_manager: Arc<CacheManager>,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) {
        let key = segment_name(namespace, shard_name, segment);
        if self.build_index_thread.contains_key(&key) {
            return;
        }
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let raw_namespace = namespace.to_string();
        let raw_shard_name = shard_name.to_string();

        tokio::spawn(async move {
            let last_offset = match get_last_offset_build_index(
                rocksdb_engine_handler,
                &raw_namespace,
                &raw_shard_name,
                segment,
            ) {
                Ok(offset) => offset,
                Err(e) => {
                    error!(
                        "Failed to get last_offset_build_index with error message :{}",
                        e
                    );
                    return;
                }
            };

            let (segment_write, _) = match open_segment_write(
                cache_manager.clone(),
                &raw_namespace,
                &raw_shard_name,
                segment,
            )
            .await
            {
                Ok((segment_write, max_size)) => (segment_write, max_size),
                Err(e) => {
                    error!("Failed to open Segment file with error message :{}", e);
                    return;
                }
            };
            let size = 10 * 1024 * 1024;
            let mut data_empty_times = 0;
            let max_data_empty_times = 10 * 60;
            loop {
                match segment_write.read(last_offset, size).await {
                    Ok(datas) => {
                        if datas.is_empty() {
                            sleep(Duration::from_secs(1)).await;
                            data_empty_times += 1;
                            // If the Segment has not written data for 10 minutes
                            // the indexing thread will exit and wait for data before continuing the build. Avoid idle threads.
                            if data_empty_times >= max_data_empty_times {
                                info!("");
                                break;
                            }
                            continue;
                        }
                        data_empty_times = 0;
                        for record in datas {
                            // build position index

                            // build timestamp index

                            // build tag index
                        }
                    }
                    Err(e) => {
                        error!("Failed to read Segment file data with error message :{}", e);
                    }
                }
            }
        });
    }
}

pub fn save_finish_build_index(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    namespace: &str,
    shard_name: &str,
    segment: u32,
) -> Result<(), JournalServerError> {
    let key = finish_build_index(namespace, shard_name, segment);
    Ok(rocksdb_engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_INDEX,
        key,
        true,
    )?)
}

pub fn is_finish_build_index(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    namespace: &str,
    shard_name: &str,
    segment: u32,
) -> Result<bool, JournalServerError> {
    let key = finish_build_index(namespace, shard_name, segment);
    Ok(rocksdb_engine_exists(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_INDEX,
        key,
    )?)
}

pub fn save_last_offset_build_index(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    namespace: &str,
    shard_name: &str,
    segment: u32,
) -> Result<(), JournalServerError> {
    let key = last_offset_build_index(namespace, shard_name, segment);
    Ok(rocksdb_engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_INDEX,
        key,
        true,
    )?)
}

pub fn get_last_offset_build_index(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    namespace: &str,
    shard_name: &str,
    segment: u32,
) -> Result<Option<u64>, JournalServerError> {
    let key = last_offset_build_index(namespace, shard_name, segment);
    if let Some(res) =
        rocksdb_engine_get(rocksdb_engine_handler.clone(), DB_COLUMN_FAMILY_INDEX, key)?
    {
        return Ok(Some(serde_json::from_slice::<u64>(&res.data)?));
    }

    Ok(None)
}

pub fn delete_segment_index(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    namespace: &str,
    shard_name: &str,
    segment: u32,
) -> Result<(), JournalServerError> {
    let prefix_key_name = segment_index_prefix(namespace, shard_name, segment);
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
