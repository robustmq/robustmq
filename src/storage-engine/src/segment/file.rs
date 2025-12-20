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

use super::SegmentIdentity;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use bytes::BytesMut;
use common_base::tools::{file_exists, try_create_fold};
use common_config::broker::broker_config;
use prost::Message;
use protocol::storage::storage_engine_record::StorageEngineRecord;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// The record read from the segment file
#[derive(Debug, Clone)]
pub struct ReadData {
    pub position: u64,
    pub record: StorageEngineRecord,
}

/// Given a segment identity, open a segment file for reading and writing.
pub async fn open_segment_write(
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
) -> Result<SegmentFile, StorageEngineError> {
    let segment = if let Some(segment) = cache_manager.get_segment(segment_iden) {
        segment
    } else {
        return Err(StorageEngineError::SegmentNotExist(segment_iden.name()));
    };

    let conf = broker_config();
    let fold = if let Some(fold) = segment.get_fold(conf.broker_id) {
        fold
    } else {
        return Err(StorageEngineError::SegmentDataDirectoryNotFound(
            segment_iden.name(),
            conf.broker_id,
        ));
    };

    Ok(SegmentFile::new(
        segment_iden.shard_name.to_string(),
        segment_iden.segment_seq,
        fold,
    ))
}

/// Represent a segment file, providing methods for reading and writing records.
#[derive(Default, Clone)]
pub struct SegmentFile {
    pub shard_name: String,
    pub segment_no: u32,
    pub data_fold: String,
}

impl SegmentFile {
    pub fn new(shard_name: String, segment_no: u32, data_fold: String) -> Self {
        let data_fold = data_fold_shard(&shard_name, &data_fold);
        SegmentFile {
            shard_name,
            segment_no,
            data_fold,
        }
    }

    /// try create a segment file under the data folder
    pub async fn try_create(&self) -> Result<(), StorageEngineError> {
        try_create_fold(&self.data_fold)?;
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        if file_exists(&segment_file) {
            return Ok(());
        }
        File::create(segment_file).await?;
        Ok(())
    }

    /// delete the segment file
    pub async fn delete(&self) -> Result<(), StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        if !file_exists(&segment_file) {
            return Err(StorageEngineError::SegmentFileNotExists(segment_file));
        }

        Ok(remove_file(segment_file)?)
    }

    /// append a list of records to the segment file
    pub async fn write(&self, records: &[StorageEngineRecord]) -> Result<(), StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = OpenOptions::new().append(true).open(segment_file).await?;
        let mut writer = tokio::io::BufWriter::new(file);

        for record in records {
            let data = StorageEngineRecord::encode_to_vec(record);
            writer.write_u64(record.offset as u64).await?;
            writer.write_u32(data.len() as u32).await?;
            writer.write_all(data.as_ref()).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    /// get the size of the segment file
    pub async fn size(&self) -> Result<u64, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let metadata = fs::metadata(segment_file).await?;
        Ok(metadata.len())
    }

    /// read a list of records starting from the byte position `start_position` in the segment file
    ///
    /// All records being returned satisfy the following conditions:
    ///     1. the offset of the record is greater than or equal to `start_offset`
    ///     2. the total size of the records is less than or equal to `max_size`
    ///     3. the number of records is less than or equal to `max_record`
    ///
    /// The records are stored in the segment file in the following format:
    ///
    ///     [offset: u64][len: u32][data: bytes]
    ///
    /// We only consider `data` when calculating the size of a record.
    ///
    /// # Return
    ///
    /// A list of records and their byte positions in the segment file, in the order in which they are stored in the segment file.
    ///
    pub async fn read_by_offset(
        &self,
        start_position: u64,
        start_offset: u64,
        max_size: u64,
        max_record: u64,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        reader
            .seek(std::io::SeekFrom::Current(start_position as i64))
            .await?;

        let mut results = Vec::new();
        let mut already_size = 0;
        loop {
            if already_size > max_size {
                break;
            }

            // read offset
            let position = reader.stream_position().await?;

            let record_offset = match reader.read_u64().await {
                Ok(offset) => offset,
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(e.into());
                }
            };

            // read len
            let len = reader.read_u32().await?;

            if record_offset < start_offset {
                reader.seek(std::io::SeekFrom::Current(len as i64)).await?;
                continue;
            }

            // read body
            let mut buf = BytesMut::with_capacity(len as usize);
            reader.read_buf(&mut buf).await?;

            already_size += buf.len() as u64;
            let record = StorageEngineRecord::decode(buf)?;
            results.push(ReadData { position, record });

            if results.len() >= max_record as usize {
                break;
            }
        }

        Ok(results)
    }

    /// read a list of records by their byte positions in the segment file
    ///
    /// See [`read_by_offset`] for more details.
    pub async fn read_by_positions(
        &self,
        positions: Vec<u64>,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut results = Vec::new();

        for position in positions {
            reader.seek(std::io::SeekFrom::Start(position)).await?;

            // read offset
            let _ = match reader.read_u64().await {
                Ok(offset) => offset,
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(e.into());
                }
            };

            // read len
            let len = reader.read_u32().await?;

            if len == 0 {
                continue;
            }

            // read body
            let mut buf = BytesMut::with_capacity(len as usize);
            reader.read_buf(&mut buf).await?;

            let record = StorageEngineRecord::decode(buf)?;

            results.push(ReadData { position, record });
        }

        Ok(results)
    }

    pub fn exists(&self) -> bool {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        Path::new(&segment_file).exists()
    }
}

pub fn data_fold_shard(shard_name: &str, data_fold: &str) -> String {
    format!("{data_fold}/{shard_name}")
}

pub fn data_file_segment(data_fold: &str, segment_no: u32) -> String {
    format!("{data_fold}/{segment_no}.msg")
}

#[cfg(test)]
mod tests {
    use super::{data_file_segment, data_fold_shard, open_segment_write, SegmentFile};
    use crate::core::cache::StorageCacheManager;
    use crate::core::test::{test_build_data_fold, test_build_segment};
    use crate::segment::SegmentIdentity;
    use broker_core::cache::BrokerCacheManager;
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::segment::{EngineSegment, Replica};
    use protocol::storage::storage_engine_record::StorageEngineRecord;
    use std::sync::Arc;

    #[tokio::test]
    async fn data_fold_shard_test() {
        let shard_name = "s1".to_string();
        let data_fold = "/tmp/d1".to_string();
        let segment_no = 10;
        let fold = data_fold_shard(&shard_name, &data_fold);
        assert_eq!(fold, format!("{data_fold}/{shard_name}"));
        let file = data_file_segment(&fold, segment_no);
        assert_eq!(file, format!("{fold}/{segment_no}.msg"));
    }

    #[tokio::test]
    async fn open_segment_write_test() {
        init_broker_conf_by_config(default_broker_config());
        let shard_name = "s1".to_string();
        let segment_no = 10;
        let segment_iden = SegmentIdentity {
            shard_name: shard_name.clone(),
            segment_seq: segment_no,
        };
        let segment = EngineSegment {
            shard_name,
            segment_seq: segment_no,
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: "/tmp/jl/tests".to_string(),
            }],
            ..Default::default()
        };

        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));

        let res = open_segment_write(&cache_manager, &segment_iden).await;
        assert!(res.is_err());

        cache_manager.set_segment(segment);
        let res = open_segment_write(&cache_manager, &segment_iden).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn segment_create() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        let segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment_seq,
            data_fold.first().unwrap().to_string(),
        );
        assert!(segment.try_create().await.is_ok());
        assert!(segment.try_create().await.is_ok());
        assert!(segment.exists());
        let res = segment.delete().await;
        assert!(res.is_ok());
        assert!(!segment.exists());
    }

    #[tokio::test]
    async fn segment_read_offset_test() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        let segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment_seq,
            data_fold.first().unwrap().to_string(),
        );

        segment.try_create().await.unwrap();
        for i in 0..10 {
            let value = format!("data1#-{i}");
            let record = StorageEngineRecord {
                content: value.as_bytes().to_vec(),
                create_time: now_second(),
                key: format!("k{i}"),
                shard_name: "s1".to_string(),
                offset: 1000 + i,
                segment: 1,
                tags: vec![],
                ..Default::default()
            };
            match segment.write(std::slice::from_ref(&record)).await {
                Ok(_) => {}
                Err(e) => {
                    panic!("{e:?}");
                }
            }
        }

        let res = segment.read_by_offset(0, 0, 20000, 1000).await.unwrap();
        assert_eq!(res.len(), 10);

        let res = segment.read_by_offset(0, 1005, 20000, 1000).await.unwrap();
        assert_eq!(res.len(), 5);
    }

    #[tokio::test]
    async fn segment_read_position_test() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        let segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment_seq,
            data_fold.first().unwrap().to_string(),
        );
        println!("{}", segment.data_fold);
        segment.try_create().await.unwrap();
        for i in 0..10 {
            let value = format!("data1#-{i}");
            let record = StorageEngineRecord {
                content: value.as_bytes().to_vec(),
                create_time: now_second(),
                key: format!("k{i}"),
                shard_name: "s1".to_string(),
                offset: 1000 + i,
                segment: segment.segment_no,
                tags: vec![],
                ..Default::default()
            };
            segment.write(std::slice::from_ref(&record)).await.unwrap();
        }

        let res = segment.read_by_positions(vec![0]).await.unwrap();
        assert_eq!(res.len(), 1);

        // data len = 41
        let res = segment.read_by_positions(vec![41]).await.unwrap();
        assert_eq!(res.len(), 1);

        let res = segment.read_by_positions(vec![0, 41, 82]).await.unwrap();
        assert_eq!(res.len(), 3);

        let size = segment.size().await.unwrap();
        assert!(size > 0);
    }
}
