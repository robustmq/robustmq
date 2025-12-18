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

use std::fs;
use std::path::Path;
use std::sync::Arc;

use common_config::broker::broker_config;
use dashmap::DashMap;
use metadata_struct::journal::segment::{segment_name, JournalSegment};
use rocksdb_engine::rocksdb::RocksDBEngine;
use tracing::{error, info};

use super::file::SegmentFile;
use super::SegmentIdentity;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::index::engine::storage_data_fold;
use crate::index::offset::OffsetIndexManager;
use crate::index::time::TimestampIndexManager;

/// struct that stores the metadata of a segment file.
#[derive(Clone, Default, Debug)]
pub struct SegmentFileMetadata {
    pub shard_name: String,
    pub segment_no: u32,
    pub start_offset: i64,
    pub end_offset: i64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

/// struct that manages the segment file metadata, providing methods to add, get, remove, and update segment file metadata.
pub struct SegmentFileManager {
    pub segment_files: DashMap<String, SegmentFileMetadata>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentFileManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let segment_files = DashMap::with_capacity(8);
        SegmentFileManager {
            segment_files,
            rocksdb_engine_handler,
        }
    }

    pub fn add_segment_file(&self, segment_file: SegmentFileMetadata) {
        let key = segment_name(&segment_file.shard_name, segment_file.segment_no);
        self.segment_files.insert(key, segment_file);
    }

    pub fn get_segment_file(&self, segment_iden: &SegmentIdentity) -> Option<SegmentFileMetadata> {
        if let Some(data) = self.segment_files.get(&segment_iden.name()) {
            return Some(data.clone());
        }
        None
    }

    pub fn remove_segment_file(&self, segment_iden: &SegmentIdentity) {
        self.segment_files.remove(&segment_iden.name());
    }

    pub fn get_end_offset(&self, segment_iden: &SegmentIdentity) -> Option<i64> {
        if let Some(data) = self.segment_files.get(&segment_iden.name()) {
            return Some(data.end_offset);
        }
        None
    }

    pub fn update_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: i64,
    ) -> Result<(), StorageEngineError> {
        if let Some(mut data) = self.segment_files.get_mut(&segment_iden.name()) {
            data.start_offset = start_offset;
            let offset_index = OffsetIndexManager::new(self.rocksdb_engine_handler.clone());
            offset_index.save_start_offset(segment_iden, data.end_offset as u64)?;
        }
        Ok(())
    }

    pub fn update_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
        end_offset: i64,
    ) -> Result<(), StorageEngineError> {
        if let Some(mut data) = self.segment_files.get_mut(&segment_iden.name()) {
            data.end_offset = end_offset;
            let offset_index = OffsetIndexManager::new(self.rocksdb_engine_handler.clone());
            offset_index.save_end_offset(segment_iden, data.end_offset as u64)?;
        }
        Ok(())
    }

    pub fn update_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        timestamp: u64,
    ) -> Result<(), StorageEngineError> {
        if let Some(mut data) = self.segment_files.get_mut(&segment_iden.name()) {
            data.start_timestamp = timestamp as i64;
            let timestamp_index = TimestampIndexManager::new(self.rocksdb_engine_handler.clone());
            timestamp_index.save_start_timestamp(segment_iden, timestamp)?;
        }
        Ok(())
    }

    pub fn update_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        timestamp: u64,
    ) -> Result<(), StorageEngineError> {
        if let Some(mut data) = self.segment_files.get_mut(&segment_iden.name()) {
            data.end_timestamp = timestamp as i64;
            let timestamp_index = TimestampIndexManager::new(self.rocksdb_engine_handler.clone());
            timestamp_index.save_end_timestamp(segment_iden, timestamp)?;
        }
        Ok(())
    }
}

/// Populate `segment_file_manager` with the segment file metadata loaded from directory `dir` and its subdirectories.
///
/// This method is called when the node starts up.
pub fn load_local_segment_cache(
    dir: &Path,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    local_data_folds: &Vec<String>,
) -> Result<(), StorageEngineError> {
    let dir_str = dir.display().to_string();
    let rocksdb_dir = storage_data_fold(local_data_folds);
    if dir_str == rocksdb_dir {
        return Ok(());
    }

    let offset_manager = OffsetIndexManager::new(rocksdb_engine_handler.clone());
    let timestamp_manager = TimestampIndexManager::new(rocksdb_engine_handler.clone());
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            load_local_segment_cache(
                &path,
                rocksdb_engine_handler,
                segment_file_manager,
                local_data_folds,
            )?;
        } else {
            let mut tmp_dir = dir_str.clone();
            for data_path in local_data_folds.clone() {
                tmp_dir = tmp_dir.replace(&data_path, "");
            }
            if tmp_dir.starts_with('/') {
                tmp_dir = tmp_dir[1..].to_string();
            }
            let tmp_dir_slice: Vec<String> =
                tmp_dir.split("/").map(|raw| raw.to_string()).collect();
            if tmp_dir_slice.len() != 2 {
                error!(
                    "Segment data directory file format error, directory name: {}",
                    dir_str
                );
                continue;
            }
            let shard_name = tmp_dir_slice.first().unwrap();

            let file_path = path.display().to_string();
            let segment_file = file_path.split("/").last().unwrap();
            let segment = segment_file.replace(".msg", "");
            let segment_no = segment.parse::<u32>()?;

            let segment_iden = SegmentIdentity {
                shard_name: shard_name.to_string(),
                segment_seq: segment_no,
            };

            let start_offset = offset_manager.get_start_offset(&segment_iden)?;
            let end_offset = offset_manager.get_end_offset(&segment_iden)?;
            let start_timestamp = timestamp_manager.get_start_timestamp(&segment_iden)?;
            let end_timestamp = timestamp_manager.get_end_timestamp(&segment_iden)?;

            let metadata = SegmentFileMetadata {
                shard_name: shard_name.to_string(),
                segment_no,
                start_offset,
                end_offset,
                start_timestamp,
                end_timestamp,
            };

            segment_file_manager.add_segment_file(metadata);
        }
    }
    Ok(())
}

pub fn metadata_and_local_segment_diff_check() {
    //todo
}

/// Create a new local segment file from `JournalSegment`.
pub async fn create_local_segment(
    cache_manager: &Arc<StorageCacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment: &JournalSegment,
) -> Result<(), StorageEngineError> {
    let segment_iden = SegmentIdentity {
        shard_name: segment.shard_name.clone(),
        segment_seq: segment.segment_seq,
    };

    if cache_manager.get_segment(&segment_iden).is_some() {
        return Ok(());
    }

    let conf = broker_config();
    let fold = if let Some(fold) = segment.get_fold(conf.broker_id) {
        fold
    } else {
        return Err(StorageEngineError::SegmentDataDirectoryNotFound(
            segment_iden.name(),
            conf.broker_id,
        ));
    };

    // create segment file
    let segment_file = SegmentFile::new(segment.shard_name.clone(), segment.segment_seq, fold);
    segment_file.try_create().await?;

    // add segment file manager
    let segment_metadata = SegmentFileMetadata {
        shard_name: segment.shard_name.clone(),
        segment_no: segment.segment_seq,
        start_offset: -1,
        end_offset: -1,
        start_timestamp: -1,
        end_timestamp: -1,
    };
    segment_file_manager.add_segment_file(segment_metadata);

    // add cache
    cache_manager.set_segment(segment.clone());

    info!("Segment {} created successfully", segment_iden.name());
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use common_base::tools::now_second;

    use super::{SegmentFileManager, SegmentFileMetadata};
    use crate::core::test::{test_build_rocksdb_sgement, test_init_segment};
    use crate::segment::file::SegmentFile;

    #[tokio::test]
    async fn segment_metadata_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        let segment_file = SegmentFileMetadata {
            shard_name: segment_iden.shard_name.to_string(),
            segment_no: segment_iden.segment_seq,
            ..Default::default()
        };
        segment_file_manager.add_segment_file(segment_file);

        let file = segment_file_manager.get_segment_file(&segment_iden);
        assert!(file.is_some());

        let start_offset = 10;
        let res = segment_file_manager.update_start_offset(&segment_iden, start_offset);
        assert!(res.is_ok());

        let end_offset = 50;
        let res = segment_file_manager.update_end_offset(&segment_iden, end_offset);
        assert!(res.is_ok());

        let start_timestamp = now_second() - 1000;
        let res = segment_file_manager.update_start_timestamp(&segment_iden, start_timestamp);
        assert!(res.is_ok());

        let end_timestamp = now_second() + 1000;
        let res = segment_file_manager.update_end_timestamp(&segment_iden, end_timestamp);
        assert!(res.is_ok());

        let file = segment_file_manager
            .get_segment_file(&segment_iden)
            .unwrap();
        assert_eq!(file.start_offset, start_offset);
        assert_eq!(file.end_offset, end_offset);
        assert_eq!(file.start_timestamp, start_timestamp as i64);
        assert_eq!(file.end_timestamp, end_timestamp as i64);

        segment_file_manager.remove_segment_file(&segment_iden);

        let file = segment_file_manager.get_segment_file(&segment_iden);
        assert!(file.is_none());
    }

    #[tokio::test]
    async fn try_create_local_segment_test() {
        let (segment_iden, _, segment_file_manager, fold, _) = test_init_segment().await;

        let res = segment_file_manager.get_segment_file(&segment_iden);
        assert!(res.is_some());
        let data = res.unwrap();
        assert_eq!(data.start_offset, -1);
        assert_eq!(data.end_offset, -1);
        assert_eq!(data.start_timestamp, -1);
        assert_eq!(data.end_timestamp, -1);

        let segment_write = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            fold,
        );
        assert!(segment_write.exists());
    }

    #[tokio::test]
    async fn load_local_segment_cache_test() {}
}
