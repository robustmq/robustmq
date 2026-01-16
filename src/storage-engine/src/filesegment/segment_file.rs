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
use bytes::{Bytes, BytesMut};
use common_base::tools::{file_exists, try_create_fold};
use common_config::broker::broker_config;
use memmap2::Mmap;
use metadata_struct::storage::storage_record::{StorageRecord, StorageRecordMetadata};
use std::collections::HashMap;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader};

// Mmap thresholds - kept for potential future use
// const MMAP_THRESHOLD: u64 = 10 * 1024 * 1024;
// const LARGE_FILE_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;

/// The record read from the segment file
#[derive(Debug, Clone)]
pub struct ReadData {
    pub position: u64,
    pub record: StorageRecord,
}

#[derive(Debug, Clone)]
struct MmapWrapper {
    mmap: Arc<Mmap>,
    file_size: u64,
}

impl MmapWrapper {
    fn new(mmap: Mmap, file_size: u64) -> Self {
        Self {
            mmap: Arc::new(mmap),
            file_size,
        }
    }

    fn read_at(&self, position: u64, len: usize) -> Result<&[u8], StorageEngineError> {
        let end = position
            .checked_add(len as u64)
            .ok_or_else(|| StorageEngineError::CommonErrorStr("Position overflow".to_string()))?;

        if end > self.file_size {
            return Err(StorageEngineError::CommonErrorStr(
                "Read beyond file size".to_string(),
            ));
        }

        Ok(&self.mmap[position as usize..end as usize])
    }

    fn read_u64_at(&self, position: u64) -> Result<u64, StorageEngineError> {
        let bytes = self.read_at(position, 8)?;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn read_u32_at(&self, position: u64) -> Result<u32, StorageEngineError> {
        let bytes = self.read_at(position, 4)?;
        Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
    }
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

    let segment_file = SegmentFile::new(
        segment_iden.shard_name.to_string(),
        segment_iden.segment,
        fold,
    )
    .await?;

    Ok(segment_file)
}

/// Represent a segment file, providing methods for reading and writing records.
#[derive(Default, Clone)]
pub struct SegmentFile {
    pub shard_name: String,
    pub segment_no: u32,
    pub data_fold: String,
    pub position: u64,
    mmap_cache: Option<MmapWrapper>,
    mmap_enabled: bool,
}

impl SegmentFile {
    pub async fn new(
        shard_name: String,
        segment_no: u32,
        data_fold: String,
    ) -> Result<Self, StorageEngineError> {
        let data_fold = data_fold_shard(&shard_name, &data_fold);
        let segment_file = data_file_segment(&data_fold, segment_no);
        try_create_fold(&data_fold)?;
        let position = fs::metadata(&segment_file)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        Ok(SegmentFile {
            shard_name,
            segment_no,
            data_fold,
            position,
            mmap_cache: None,
            mmap_enabled: true,
        })
    }

    /// try create a segment file under the data folder
    pub async fn try_create(&self) -> Result<(), StorageEngineError> {
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
    pub async fn write(
        &mut self,
        records: &[StorageRecord],
    ) -> Result<HashMap<u64, u64>, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = OpenOptions::new().append(true).open(segment_file).await?;
        let mut writer = tokio::io::BufWriter::new(file);

        // offset + total_len + metadata_len + metadata + data_len + data
        let mut offset_positions = HashMap::new();
        for record in records {
            let metadata_bytes = record.metadata.encode();
            let metadata_bytes_len = metadata_bytes.len();
            let data_len = record.data.len();
            let total_len = data_len + metadata_bytes_len;
            offset_positions.insert(record.metadata.offset, self.position);

            writer.write_u64(record.metadata.offset).await?;
            writer.write_u32(total_len as u32).await?;
            writer.write_u32(metadata_bytes_len as u32).await?;
            writer.write_all(metadata_bytes.as_ref()).await?;
            writer.write_u32(data_len as u32).await?;
            writer.write_all(record.data.as_ref()).await?;

            // record len
            self.position += (8 + 4 + 4 + metadata_bytes_len + 4 + data_len) as u64;
        }
        writer.flush().await?;
        Ok(offset_positions)
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
        &mut self,
        start_position: u64,
        start_offset: u64,
        max_size: u64,
        max_record: u64,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        // Whether to enable mmap can be configured based on the small and large parameters.
        if self.mmap_enabled {
            self.ensure_mmap().await?;

            if let Some(ref mmap) = self.mmap_cache {
                return self.read_by_offset_mmap(mmap, start_position, start_offset, max_size, max_record);
            }
        }

        self.read_by_offset_traditional(start_position, start_offset, max_size, max_record).await
    }

    fn read_by_offset_mmap(
        &self,
        mmap: &MmapWrapper,
        start_position: u64,
        start_offset: u64,
        max_size: u64,
        max_record: u64,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let mut results = Vec::new();
        let mut already_size = 0;
        let mut pos = start_position;

        // Get file size for boundary check
        let file_size = mmap.file_size;

        loop {
            if results.len() >= max_record as usize {
                break;
            }

            // Check if we have enough bytes for the header (offset + total_len + metadata_len)
            // Minimum header size: 8 + 4 + 4 = 16 bytes
            if pos + 16 > file_size {
                break;
            }

            // read offset (8 bytes) and total_len (4 bytes)
            let offset = mmap.read_u64_at(pos)?;
            let total_len = mmap.read_u32_at(pos + 8)?;

            if offset < start_offset {
                pos += 20 + total_len as u64;
                continue;
            }

            // Check if we can read the full record.
            // File format: offset(8) + total_len(4) + metadata_len(4) + metadata + data_len(4) + data
            // total_len = metadata_len + data_len
            // So full record size = 8 + 4 + 4 + total_len + 4 = 20 + total_len
            // Boundary check: pos + 16 (offset+total_len+metadata_len) + total_len <= file_size
            if pos + 16 + total_len as u64 > file_size {
                break;
            }

            // read record at position + 12 (skip offset and total_len)
            let record = self.read_record_mmap(mmap, pos)?;

            let data_size = record.data.len() as u64;
            if already_size + data_size > max_size {
                break;
            }
            already_size += data_size;

            results.push(ReadData {
                record,
                position: pos,
            });

            pos += 20 + total_len as u64;
        }

        Ok(results)
    }

    async fn read_by_offset_traditional(
        &self,
        start_position: u64,
        start_offset: u64,
        max_size: u64,
        max_record: u64,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader: tokio::io::BufReader<File> = tokio::io::BufReader::new(file);

        reader
            .seek(std::io::SeekFrom::Current(start_position as i64))
            .await?;

        let mut results = Vec::new();
        let mut already_size = 0;
        loop {
            if results.len() >= max_record as usize {
                break;
            }

            let position = reader.stream_position().await?;

            // read offset
            let offset = match reader.read_u64().await {
                Ok(offset) => offset,
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => {
                        return Err(e.into());
                    }
                },
            };

            // read total len
            let total_len = reader.read_u32().await?;

            if offset < start_offset {
                reader
                    .seek(std::io::SeekFrom::Current(total_len as i64 + 8))
                    .await?;
                continue;
            }

            // read data
            let data = self.read_data(&mut reader).await?;
            if let Some(da) = data {
                let data_size = da.data.len() as u64;
                if already_size + data_size > max_size {
                    break;
                }
                already_size += data_size;
                results.push(ReadData {
                    record: da,
                    position,
                });
            }
        }

        Ok(results)
    }

    async fn ensure_mmap(&mut self) -> Result<(), StorageEngineError> {
        if self.mmap_cache.is_some() {
            return Ok(());
        }

        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file_size = match fs::metadata(&segment_file).await {
            Ok(meta) => meta.len(),
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        // For small files, use mmap as well - the overhead is minimal and it simplifies the code
        // For very large files, we may want to use a different strategy in the future

        // if file_size > LARGE_FILE_THRESHOLD {
        //     todo!("large file: consider using range mmap or direct I/O");
        // }

        let segment_file_clone = segment_file.clone();
        let mmap = tokio::task::spawn_blocking(move || -> Result<Mmap, StorageEngineError> {
            let file = std::fs::File::open(segment_file_clone)?;
            let mmap = unsafe { Mmap::map(&file)? };
            Ok(mmap)
        })
        .await
        .map_err(|e| {
            StorageEngineError::CommonErrorStr(format!("Spawn blocking error: {}", e))
        })??;

        self.mmap_cache = Some(MmapWrapper::new(mmap, file_size));
        Ok(())
    }

    fn read_record_mmap(
        &self,
        mmap: &MmapWrapper,
        position: u64,
    ) -> Result<StorageRecord, StorageEngineError> {
        let mut pos = position;

        pos += 12;

        let metadata_len = mmap.read_u32_at(pos)? as usize;
        pos += 4;

        // decode need aligned bytes
        let metadata_bytes = mmap.read_at(pos, metadata_len)?;
        let metadata_bytes = Bytes::copy_from_slice(metadata_bytes);
        let metadata = StorageRecordMetadata::decode(&metadata_bytes).map_err(|e| {
            StorageEngineError::CommonErrorStr(format!(
                "Failed to decode metadata in segment file for shard {}, segment {}: {}",
                self.shard_name, self.segment_no, e
            ))
        })?;
        pos += metadata_len as u64;

        let data_len = mmap.read_u32_at(pos)? as usize;
        pos += 4;

        let data_bytes = mmap.read_at(pos, data_len)?;
        let data = Bytes::copy_from_slice(data_bytes);

        Ok(StorageRecord { metadata, data })
    }

    /// read a list of records by their byte positions in the segment file
    ///
    /// See [`read_by_offset`] for more details.
    pub async fn read_by_positions(
        &mut self,
        positions: Vec<u64>,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        // Whether to enable mmap can be configured based on the small and large parameters.
        if self.mmap_enabled {
            self.ensure_mmap().await?;

            if let Some(ref mmap) = self.mmap_cache {
                return self.read_by_positions_mmap(mmap, positions);
            }
        }

        self.read_by_positions_traditional(positions).await
    }

    fn read_by_positions_mmap(
        &self,
        mmap: &MmapWrapper,
        positions: Vec<u64>,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let mut results = Vec::with_capacity(positions.len());

        for position in positions {
            let record = self.read_record_mmap(mmap, position)?;
            results.push(ReadData { position, record });
        }

        Ok(results)
    }

    async fn read_by_positions_traditional(
        &self,
        positions: Vec<u64>,
    ) -> Result<Vec<ReadData>, StorageEngineError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut results = Vec::new();

        for position in positions {
            reader.seek(std::io::SeekFrom::Start(position)).await?;

            // skip offset(8 bytes) + total_len(4 bytes) = 12 bytes
            reader.seek(std::io::SeekFrom::Current(12)).await?;

            // read data
            let data = self.read_data(&mut reader).await?;
            if let Some(da) = data {
                results.push(ReadData {
                    position,
                    record: da,
                });
            }
        }

        Ok(results)
    }

    pub fn exists(&self) -> bool {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        Path::new(&segment_file).exists()
    }

    async fn read_data(
        &self,
        reader: &mut BufReader<File>,
    ) -> Result<Option<StorageRecord>, StorageEngineError> {
        // read metadata len
        let metadata_len = reader.read_u32().await?;

        // read metadata
        let mut metadata_buf = BytesMut::with_capacity(metadata_len as usize);
        reader.read_buf(&mut metadata_buf).await?;

        let metadata = match StorageRecordMetadata::decode(metadata_buf.as_ref()) {
            Ok(data) => data,
            Err(e) => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Failed to decode metadata in segment file for shard {}, segment {}: {}",
                    self.shard_name, self.segment_no, e
                )))
            }
        };

        // read data len
        let data_len = reader.read_u32().await?;

        // read data
        let mut data_buf = BytesMut::with_capacity(data_len as usize);
        reader.read_buf(&mut data_buf).await?;

        Ok(Some(StorageRecord {
            metadata,
            data: data_buf.into(),
        }))
    }

    pub fn clear_cache(&mut self) {
        self.mmap_cache = None;
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
    use crate::core::error::StorageEngineError;
    use crate::core::test_tool::{test_build_data_fold, test_build_segment};
    use crate::filesegment::SegmentIdentity;
    use broker_core::cache::BrokerCacheManager;
    use bytes::Bytes;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::segment::{EngineSegment, Replica};
    use metadata_struct::storage::storage_record::{StorageRecord, StorageRecordMetadata};
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
            segment: segment_no,
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

        cache_manager.set_segment(&segment);
        let res = open_segment_write(&cache_manager, &segment_iden).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn segment_create() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        let segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();
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

        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();
        for i in 0..10 {
            let value = format!("data1#-{i}");
            let data = Bytes::from(value);
            let record = StorageRecord {
                metadata: StorageRecordMetadata::new(
                    1000 + i,
                    &segment_iden.shard_name,
                    segment_iden.segment,
                    &None,
                    &None,
                    &None,
                    &data,
                ),
                data,
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

        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();
        let mut all_positions = Vec::new();
        for i in 0..10 {
            let value = format!("data1#-{i}");
            let data = Bytes::from(value);
            let record = StorageRecord {
                metadata: StorageRecordMetadata::new(
                    1000 + i,
                    &segment_iden.shard_name,
                    segment_iden.segment,
                    &None,
                    &None,
                    &None,
                    &data,
                ),
                data,
            };
            let positions = segment.write(std::slice::from_ref(&record)).await.unwrap();
            if let Some(pos) = positions.get(&(1000 + i)) {
                all_positions.push(*pos);
            }
        }

        let res = segment
            .read_by_positions(vec![all_positions[0]])
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        let res = segment
            .read_by_positions(vec![all_positions[1]])
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        let res = segment
            .read_by_positions(vec![all_positions[0], all_positions[1], all_positions[2]])
            .await
            .unwrap();
        assert_eq!(res.len(), 3);

        let size = segment.size().await.unwrap();
        assert!(size > 0);
    }

    #[tokio::test]
    async fn segment_boundary_check_test() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();
        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();
        for i in 0..10 {
            let data = Bytes::from(vec![0u8; 100]);
            let record = StorageRecord {
                metadata: StorageRecordMetadata::new(
                    100 + i,
                    &segment_iden.shard_name,
                    segment_iden.segment,
                    &None,
                    &None,
                    &None,
                    &data,
                ),
                data,
            };
            segment.write(std::slice::from_ref(&record)).await.unwrap();
        }

        let res = segment.read_by_offset(0, 0, 150, 1000).await.unwrap();
        assert_eq!(res.len(), 1);

        let res = segment.read_by_offset(0, 0, 100000, 3).await.unwrap();
        assert_eq!(res.len(), 3);
    }

    #[tokio::test]
    async fn segment_error_handling_test() {
        let data_fold = test_build_data_fold();
        let mut segment = SegmentFile::new(
            "test_shard".to_string(),
            999,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        let res = segment.read_by_offset(0, 0, 1000, 10).await;
        assert!(res.is_err());

        let res = segment.read_by_positions(vec![0]).await;
        assert!(res.is_err());

        let res = segment.delete().await;
        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            StorageEngineError::SegmentFileNotExists(_)
        ));
    }

    #[tokio::test]
    async fn segment_data_integrity_test() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();
        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();
        let data = Bytes::from("test-data-content");
        let record = StorageRecord {
            metadata: StorageRecordMetadata::build(
                100,
                segment_iden.shard_name.to_string(),
                segment_iden.segment,
            )
            .with_key(Some("test-key".to_string()))
            .with_tags(Some(vec!["tag1".to_string(), "tag2".to_string()]))
            .with_timestamp(12345)
            .with_crc_from_data(&data),
            data,
        };

        segment.write(&[record]).await.unwrap();
        let res = segment.read_by_offset(0, 0, 10000, 10).await.unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].record.metadata.offset, 100);
        assert_eq!(res[0].record.metadata.key, Some("test-key".to_string()));
        assert_eq!(
            res[0].record.metadata.tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
        assert_eq!(res[0].record.data.as_ref(), b"test-data-content");
        assert_eq!(res[0].record.metadata.create_t, 12345);
    }

    #[tokio::test]
    async fn segment_edge_cases_test() {
        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();
        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();

        let res = segment.read_by_offset(0, 0, 1000, 10).await.unwrap();
        assert_eq!(res.len(), 0);

        let res = segment.read_by_positions(vec![]).await.unwrap();
        assert_eq!(res.len(), 0);

        segment.write(&[]).await.unwrap();
    }

    // ============================================================
    // Performance Benchmark Tests for Mmap vs Traditional IO
    // ============================================================

    /// Benchmark: Compare mmap vs traditional IO read performance
    /// Run with: cargo test --release -- segment_file::tests::bench_read_performance --nocapture
    #[tokio::test]
    async fn bench_read_performance() {
        use std::time::{Duration, Instant};

        // Test configuration
        const RECORD_COUNT: usize = 1000;
        const DATA_SIZE_PER_RECORD: usize = 1024; // 1KB per record

        let data_fold = test_build_data_fold();
        let segment_iden = test_build_segment();

        // Create segment and write test data
        let mut segment = SegmentFile::new(
            segment_iden.shard_name.to_string(),
            segment_iden.segment,
            data_fold.first().unwrap().to_string(),
        )
        .await
        .unwrap();

        segment.try_create().await.unwrap();

        let mut positions = Vec::new();
        let mut all_offsets = Vec::new();

        // Write records
        for i in 0..RECORD_COUNT {
            let data = Bytes::from(vec![0u8; DATA_SIZE_PER_RECORD]);
            let record = StorageRecord {
                metadata: StorageRecordMetadata::new(
                    i as u64,
                    &segment_iden.shard_name,
                    segment_iden.segment,
                    &None,
                    &None,
                    &None,
                    &data,
                ),
                data,
            };
            let pos_map = segment.write(std::slice::from_ref(&record)).await.unwrap();
            if let Some(pos) = pos_map.get(&(i as u64)) {
                positions.push(*pos);
                all_offsets.push(i as u64);
            }
        }

        let file_size = segment.size().await.unwrap();
        println!("\n=== Segment File Performance Benchmark ===");
        println!(
            "File size: {} bytes ({:.2} KB)",
            file_size,
            file_size as f64 / 1024.0
        );
        println!("Record count: {}", RECORD_COUNT);
        println!("Data size per record: {} bytes", DATA_SIZE_PER_RECORD);
        println!("==========================================\n");

        // Warm up mmap by (this initializes the mmap cache)
        // let _ = segment.ensure_mmap().await;

        // Benchmark 1: Random read by position (mmap IO path)
        println!("[Random Read by Position - Mmap IO]");
        let mut mmap_random_times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let res = segment.read_by_positions(positions.clone()).await.unwrap();
            let elapsed = start.elapsed();
            mmap_random_times.push(elapsed);
            assert_eq!(res.len(), RECORD_COUNT);
        }
        let mmap_random_avg =
            mmap_random_times.iter().sum::<Duration>() / mmap_random_times.len() as u32;
        let mmap_random_throughput =
            (file_size as f64) / mmap_random_avg.as_secs_f64() / 1024.0 / 1024.0;
        println!("  Average time: {:?}", mmap_random_avg);
        println!("  Throughput: {:.2} MB/s\n", mmap_random_throughput);

        segment.clear_cache();
        segment.mmap_enabled = false;

        // Benchmark 2: Random read by position (traditional IO path)
        println!("[Random Read by Position - Traditional IO]");
        let mut traditional_random_times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let res = segment.read_by_positions(positions.clone()).await.unwrap();
            let elapsed = start.elapsed();
            traditional_random_times.push(elapsed);
            assert_eq!(res.len(), RECORD_COUNT);
        }
        let traditional_random_avg = traditional_random_times.iter().sum::<Duration>()
            / traditional_random_times.len() as u32;
        let traditional_random_throughput =
            (file_size as f64) / traditional_random_avg.as_secs_f64() / 1024.0 / 1024.0;
        println!("  Average time: {:?}", traditional_random_avg);
        println!("  Throughput: {:.2} MB/s\n", traditional_random_throughput);

        // Summary
        println!("Random Read by Position:");
        println!(
            "  Traditional IO: {:?} ({:.2} MB/s)",
            traditional_random_avg, traditional_random_throughput
        );
        println!(
            "  Mmap IO: {:?} ({:.2} MB/s)",
            mmap_random_avg, mmap_random_throughput
        );
        let random_speedup = traditional_random_avg.as_secs_f64() / mmap_random_avg.as_secs_f64();
        println!("  Mmap speedup: {:.2}x\n", random_speedup);
        println!("============================\n");

        // ============================================================
        // Benchmark 3: Sequential read by offset (mmap IO path)
        // ============================================================
        segment.mmap_enabled = true;
        segment.clear_cache();
        // let _ = segment.ensure_mmap().await;

        println!("[Sequential Read by Offset - Mmap IO]");
        let mut mmap_sequential_times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let res = segment
                .read_by_offset(0, 0, 1024 * 1024 * 1024, RECORD_COUNT as u64)
                .await
                .unwrap();
            let elapsed = start.elapsed();
            mmap_sequential_times.push(elapsed);
            assert_eq!(res.len(), RECORD_COUNT);
        }
        let mmap_sequential_avg =
            mmap_sequential_times.iter().sum::<Duration>() / mmap_sequential_times.len() as u32;
        let mmap_sequential_throughput =
            (file_size as f64) / mmap_sequential_avg.as_secs_f64() / 1024.0 / 1024.0;
        println!("  Average time: {:?}", mmap_sequential_avg);
        println!("  Throughput: {:.2} MB/s\n", mmap_sequential_throughput);

        segment.clear_cache();
        segment.mmap_enabled = false;

        // ============================================================
        // Benchmark 4: Sequential read by offset (traditional IO path)
        // ============================================================
        println!("[Sequential Read by Offset - Traditional IO]");
        let mut traditional_sequential_times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let res = segment
                .read_by_offset(0, 0, 1024 * 1024 * 1024, RECORD_COUNT as u64)
                .await
                .unwrap();
            let elapsed = start.elapsed();
            traditional_sequential_times.push(elapsed);
            assert_eq!(res.len(), RECORD_COUNT);
        }
        let traditional_sequential_avg = traditional_sequential_times.iter().sum::<Duration>()
            / traditional_sequential_times.len() as u32;
        let traditional_sequential_throughput =
            (file_size as f64) / traditional_sequential_avg.as_secs_f64() / 1024.0 / 1024.0;
        println!("  Average time: {:?}", traditional_sequential_avg);
        println!("  Throughput: {:.2} MB/s\n", traditional_sequential_throughput);

        // Summary for sequential read
        println!("Sequential Read by Offset:");
        println!(
            "  Traditional IO: {:?} ({:.2} MB/s)",
            traditional_sequential_avg, traditional_sequential_throughput
        );
        println!(
            "  Mmap IO: {:?} ({:.2} MB/s)",
            mmap_sequential_avg, mmap_sequential_throughput
        );
        let sequential_speedup =
            traditional_sequential_avg.as_secs_f64() / mmap_sequential_avg.as_secs_f64();
        println!("  Mmap speedup: {:.2}x", sequential_speedup);
        println!("============================\n");

        // Clean up
        let _ = segment.delete().await;
    }
}
