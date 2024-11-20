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

use common_base::config::journal_server::journal_server_conf;
use dashmap::DashMap;
use log::error;
use metadata_struct::journal::segment::JournalSegment;
use rocksdb_engine::RocksDBEngine;

use super::file::SegmentFile;
use super::SegmentIdentity;
use crate::core::error::JournalServerError;
use crate::index::engine::storage_data_fold;
use crate::index::offset::OffsetIndexManager;

#[derive(Clone)]
pub struct SegmentFileMetadata {
    pub namespace: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub start_offset: i64,
    pub end_offset: i64,
}
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
        let key = self.key(
            &segment_file.namespace,
            &segment_file.shard_name,
            segment_file.segment_no,
        );
        self.segment_files.insert(key, segment_file);
    }

    pub fn get_segment_file(&self, segment_iden: &SegmentIdentity) -> Option<SegmentFileMetadata> {
        let key = self.key(
            &segment_iden.namespace,
            &segment_iden.shard_name,
            segment_iden.segment_seq,
        );
        if let Some(data) = self.segment_files.get(&key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_segment_end_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) -> Option<i64> {
        let key = self.key(namespace, shard_name, segment);
        if let Some(data) = self.segment_files.get(&key) {
            return Some(data.end_offset);
        }
        None
    }

    pub fn update_start_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        start_offset: i64,
    ) -> Result<(), JournalServerError> {
        let key = self.key(namespace, shard_name, segment);
        if let Some(mut data) = self.segment_files.get_mut(&key) {
            data.start_offset = start_offset;
            let offset_index = OffsetIndexManager::new(self.rocksdb_engine_handler.clone());
            offset_index.save_end_offset(namespace, shard_name, segment, data.end_offset as u64)?;
        }
        Ok(())
    }

    pub fn update_end_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        end_offset: i64,
    ) -> Result<(), JournalServerError> {
        let key = self.key(namespace, shard_name, segment);
        if let Some(mut data) = self.segment_files.get_mut(&key) {
            data.end_offset = end_offset;
            let offset_index = OffsetIndexManager::new(self.rocksdb_engine_handler.clone());
            offset_index.save_end_offset(namespace, shard_name, segment, data.end_offset as u64)?;
        }
        Ok(())
    }

    pub fn remove_segment_file(&self, namespace: &str, shard_name: &str, segment: u32) {
        let key = self.key(namespace, shard_name, segment);
        self.segment_files.remove(&key);
    }

    fn key(&self, namespace: &str, shard_name: &str, segment: u32) -> String {
        format!("{}_{}_{}", namespace, shard_name, segment)
    }
}

pub fn load_local_segment_cache(
    dir: &Path,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    local_data_folds: &Vec<String>,
) -> Result<(), JournalServerError> {
    let dir_str = dir.display().to_string();
    let rocksdb_dir = storage_data_fold(local_data_folds);
    if dir_str == rocksdb_dir {
        return Ok(());
    }

    let offset_manager = OffsetIndexManager::new(rocksdb_engine_handler.clone());
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
            let namespace = tmp_dir_slice.first().unwrap();
            let shard_name = tmp_dir_slice.get(1).unwrap();

            let file_path = path.display().to_string();
            let segment_file = file_path.split("/").last().unwrap();
            let segment = segment_file.replace(".msg", "");
            let segment_no = segment.parse::<u32>()?;

            let start_offset =
                offset_manager.get_start_offset(namespace, shard_name, segment_no)?;
            let end_offset = offset_manager.get_end_offset(namespace, shard_name, segment_no)?;

            let metadata = SegmentFileMetadata {
                namespace: namespace.to_string(),
                shard_name: shard_name.to_string(),
                segment_no,
                start_offset: start_offset as i64,
                end_offset: end_offset as i64,
            };
            segment_file_manager.add_segment_file(metadata);
        }
    }
    Ok(())
}

pub fn metadata_and_local_segment_diff_check() {
    //todo
}

pub async fn try_create_local_segment(
    segment_file_manager: &Arc<SegmentFileManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment: &JournalSegment,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let fold = if let Some(fold) = segment.get_fold(conf.node_id) {
        fold
    } else {
        return Err(JournalServerError::SegmentDataDirectoryNotFound(
            format!("{}-{}", segment.shard_name, segment.segment_seq),
            conf.node_id,
        ));
    };

    // create segment file
    let segment_file = SegmentFile::new(
        segment.namespace.clone(),
        segment.shard_name.clone(),
        segment.segment_seq,
        fold,
    );

    segment_file.try_create().await?;

    // add segment file manager
    let segment_iden = SegmentIdentity {
        namespace: segment.namespace.clone(),
        shard_name: segment.shard_name.clone(),
        segment_seq: segment.segment_seq,
    };

    if segment_file_manager
        .get_segment_file(&segment_iden)
        .is_none()
    {
        let segment_metadata = SegmentFileMetadata {
            namespace: segment.namespace.clone(),
            shard_name: segment.shard_name.clone(),
            segment_no: segment.segment_seq,
            start_offset: -1,
            end_offset: -1,
        };
        segment_file_manager.add_segment_file(segment_metadata);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use rocksdb_engine::RocksDBEngine;

    use super::{load_local_segment_cache, SegmentFileManager};
    use crate::index::engine::{column_family_list, storage_data_fold};

    #[tokio::test]
    async fn log_segment_cache_test() {
        let data_fold = vec!["/tmp/tests/jl".to_string()];

        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&data_fold),
            10000,
            column_family_list(),
        ));
        let segment_file_manager =
            Arc::new(SegmentFileManager::new(rocksdb_engine_handler.clone()));

        for path in data_fold.clone() {
            let path = Path::new(&path);
            load_local_segment_cache(
                path,
                &rocksdb_engine_handler,
                &segment_file_manager,
                &data_fold,
            )
            .unwrap();
        }
        println!("{}", segment_file_manager.segment_files.len());
    }
}
