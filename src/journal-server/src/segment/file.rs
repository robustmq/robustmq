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

use std::fs::remove_dir_all;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;

use bytes::BytesMut;
use common_base::config::journal_server::journal_server_conf;
use common_base::tools::{file_exists, try_create_fold};
use prost::Message;
use protocol::journal_server::journal_record::JournalRecord;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::SegmentIdentity;
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;

#[derive(Debug, Clone)]
pub struct ReadData {
    pub position: u64,
    pub record: JournalRecord,
}

pub async fn open_segment_write(
    cache_manager: &Arc<CacheManager>,
    segment_iden: &SegmentIdentity,
) -> Result<(SegmentFile, u64), JournalServerError> {
    let segment = if let Some(segment) = cache_manager.get_segment(segment_iden) {
        segment
    } else {
        return Err(JournalServerError::SegmentNotExist(segment_iden.name()));
    };

    let conf = journal_server_conf();
    let fold = if let Some(fold) = segment.get_fold(conf.node_id) {
        fold
    } else {
        return Err(JournalServerError::SegmentDataDirectoryNotFound(
            segment_iden.name(),
            conf.node_id,
        ));
    };

    Ok((
        SegmentFile::new(
            segment_iden.namespace.to_string(),
            segment_iden.shard_name.to_string(),
            segment_iden.segment_seq,
            fold,
        ),
        segment.config.max_segment_size,
    ))
}

#[derive(Default)]
pub struct SegmentFile {
    pub namespace: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub data_fold: String,
}

impl SegmentFile {
    pub fn new(namespace: String, shard_name: String, segment_no: u32, data_fold: String) -> Self {
        let data_fold = data_fold_shard(&namespace, &shard_name, &data_fold);
        SegmentFile {
            namespace,
            shard_name,
            segment_no,
            data_fold,
        }
    }

    pub async fn try_create(&self) -> Result<(), JournalServerError> {
        try_create_fold(&self.data_fold)?;
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        if file_exists(&segment_file) {
            return Ok(());
        }
        File::create(segment_file).await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<(), JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        if !file_exists(&segment_file) {
            return Err(JournalServerError::SegmentFileNotExists(segment_file));
        }
        Ok(remove_dir_all(segment_file)?)
    }

    pub async fn write(&self, records: &[JournalRecord]) -> Result<Vec<u64>, JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = OpenOptions::new().append(true).open(segment_file).await?;
        let mut writer = tokio::io::BufWriter::new(file);

        let mut results = Vec::new();
        for record in records {
            let position = writer.stream_position().await?;
            let data = JournalRecord::encode_to_vec(record);
            writer.write_u64(record.offset).await?;
            writer.write_u32(data.len() as u32).await?;
            writer.write_all(data.as_ref()).await?;
            results.push(position);
        }

        writer.flush().await?;
        Ok(results)
    }

    pub async fn size(&self) -> Result<u64, JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let metadata = fs::metadata(segment_file).await?;
        Ok(metadata.len())
    }

    pub async fn read_by_offset(
        &self,
        start_position: u64,
        start_offset: u64,
        size: u64,
    ) -> Result<Vec<ReadData>, JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        reader
            .seek(std::io::SeekFrom::Current(start_position as i64))
            .await?;

        let mut results = Vec::new();
        let mut already_size = 0;
        loop {
            if already_size > size {
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
            let record = JournalRecord::decode(buf)?;
            results.push(ReadData { position, record });
        }

        Ok(results)
    }

    pub async fn read_by_timestamp(
        &self,
        start_position: u64,
        timestamp: u64,
        size: u64,
    ) -> Result<Vec<ReadData>, JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        reader
            .seek(std::io::SeekFrom::Current(start_position as i64))
            .await?;

        let mut results = Vec::new();
        let mut already_size = 0;
        loop {
            if already_size > size {
                break;
            }

            // read offset
            let position = reader.stream_position().await?;

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
            // read body
            let mut buf = BytesMut::with_capacity(len as usize);
            reader.read_buf(&mut buf).await?;
            let buf_len = buf.len();

            let record = JournalRecord::decode(buf)?;

            if record.create_time < timestamp {
                reader
                    .seek(std::io::SeekFrom::Current(buf_len as i64))
                    .await?;
                continue;
            }

            already_size += buf_len as u64;
            results.push(ReadData { position, record });
        }

        Ok(results)
    }

    pub async fn read_by_positions(
        &self,
        positions: Vec<u64>,
        size: u64,
    ) -> Result<Vec<ReadData>, JournalServerError> {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut results = Vec::new();
        let mut already_size = 0;

        for position in positions {
            if already_size > size {
                break;
            }

            reader
                .seek(std::io::SeekFrom::Current(position as i64))
                .await?;

            // read len
            let len = reader.read_u32().await?;

            // read body
            let mut buf = BytesMut::with_capacity(len as usize);
            reader.read_buf(&mut buf).await?;

            let buf_len = buf.len();
            let record = JournalRecord::decode(buf)?;

            already_size += buf_len as u64;
            results.push(ReadData { position, record });
        }

        Ok(results)
    }

    pub fn exists(&self) -> bool {
        let segment_file = data_file_segment(&self.data_fold, self.segment_no);
        Path::new(&segment_file).exists()
    }
}

pub fn data_fold_shard(namespace: &str, shard_name: &str, data_fold: &str) -> String {
    let file_name = format!("{}/{}", namespace, shard_name);
    format!("{}/{}", data_fold, file_name)
}

pub fn data_file_segment(data_fold: &str, segment_no: u32) -> String {
    format!("{}/{}.msg", data_fold, segment_no)
}

#[cfg(test)]
mod tests {
    use common_base::tools::{now_second, unique_id};
    use protocol::journal_server::journal_record::JournalRecord;

    use super::SegmentFile;
    #[tokio::test]
    async fn segment_create() {
        let data_fold = "/tmp/jl/tests";

        let namespace = unique_id();
        let shard_name = "s1";
        let segment_no = 10;
        let segment = SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            data_fold.to_string(),
        );
        assert!(segment.try_create().await.is_ok());
        assert!(segment.try_create().await.is_ok());
    }

    #[tokio::test]
    async fn segment_rw_test() {
        let data_fold = "/tmp/jl/tests";

        let namespace = unique_id();
        let shard_name = "s1";
        let segment_no = 10;

        let segment = SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            data_fold.to_string(),
        );

        segment.try_create().await.unwrap();
        for i in 0..10 {
            let value = format!("data1#-{}", i);
            let record = JournalRecord {
                content: value.as_bytes().to_vec(),
                create_time: now_second(),
                key: format!("k{}", i),
                namespace: "n1".to_string(),
                shard_name: "s1".to_string(),
                offset: 1000 + i,
                segment: 1,
                tags: vec![],
            };
            match segment.write(&[record.clone()]).await {
                Ok(_) => {}
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }

        let res = segment.read_by_offset(0, 0, 20000).await.unwrap();
        for raw in res {
            println!("{:?}", raw);
        }
    }
}
