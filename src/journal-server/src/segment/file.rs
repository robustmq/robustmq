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

use common_base::tools::{file_exists, try_create_fold};
use prost::Message;
use protocol::journal_server::journal_record::JournalRecord;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::br_recode_vec;
use crate::core::error::JournalServerError;

#[derive(Default)]
pub struct SegmentFile {
    pub namespace: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub data_fold: String,
}

impl SegmentFile {
    pub fn new(namespace: String, shard_name: String, segment_no: u32, data_fold: String) -> Self {
        let data_fold = SegmentFile::build_shard_fold(&namespace, &shard_name, &data_fold);
        SegmentFile {
            namespace,
            shard_name,
            segment_no,
            data_fold,
        }
    }

    async fn create(&self) -> Result<File, JournalServerError> {
        try_create_fold(&self.data_fold)?;
        let segment_file = self.build_segment_file_path();
        if file_exists(&segment_file) {
            return Err(JournalServerError::SegmentFileAlreadyExists(segment_file));
        }
        Ok(File::create(segment_file).await?)
    }

    pub async fn write(&self, record: JournalRecord) -> Result<(), JournalServerError> {
        let segment_file = self.build_segment_file_path();
        let file = OpenOptions::new().append(true).open(segment_file).await?;

        let mut data = JournalRecord::encode_to_vec(&record);
        data.extend(br_recode_vec());
        let mut writer = tokio::io::BufWriter::new(file);
        writer.write_all(data.as_ref()).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn read(
        &self,
        offset: Option<u64>,
        size: u64,
    ) -> Result<Vec<JournalRecord>, JournalServerError> {
        let segment_file = self.build_segment_file_path();
        let file = File::open(segment_file).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut results = Vec::new();

        let mut data = Vec::new();
        reader.read_until(b'\n', &mut data).await?;
        println!("size:{}", data.len());

        // remove \n
        let end = data.len() - 1;
        let resp = &data[0..end];

        let record = JournalRecord::decode(resp.as_ref())?;
        results.push(record);

        Ok(results)
    }

    fn build_shard_fold(namespace: &str, shard_name: &str, data_fold: &str) -> String {
        let file_name = format!("{}/{}", namespace, shard_name);
        format!("{}/{}", data_fold, file_name)
    }
    fn build_segment_file_path(&self) -> String {
        format!("{}/{}.msg", self.data_fold, self.segment_no)
    }

    fn seek_offset(&self, offfset: u64) -> Result<u64, JournalServerError> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;

    use common_base::tools::{file_exists, now_second};
    use prost::Message;
    use protocol::journal_server::journal_record::JournalRecord;

    use super::SegmentFile;
    use crate::segment::br_recode_vec;
    #[tokio::test]
    async fn segment_create() {
        let namespace = "segment_create_v1";
        let shard_name = "s1";
        let segment_no = 10;
        let data_fold = "/tmp/tests/jl";
        let segment = SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            data_fold.to_string(),
        );
        assert!(segment.create().await.is_ok());
        assert!(segment.create().await.is_err());
        remove_dir_all(data_fold).unwrap();
    }

    #[tokio::test]
    async fn segment_rw_test() {
        let namespace = "segment_write_V1";
        let shard_name = "s1";
        let segment_no = 10;
        let data_fold = "/tmp/tests/jl";
        if file_exists(data_fold) {
            remove_dir_all(&data_fold).unwrap();
        }

        let segment = SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            data_fold.to_string(),
        );

        segment.create().await.unwrap();

        let value = format!("data1-{}", 1);
        let record = JournalRecord {
            content: value.as_bytes().to_vec(),
            create_time: now_second(),
            key: "k1".to_string(),
            namespace: "n1".to_string(),
            shard_name: "s1".to_string(),
            offset: 1000,
            segment: 1,
            tag: "t1".to_string(),
        };
        match segment.write(record).await {
            Ok(()) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // let res = segment.read(Some(0), 0).await.unwrap();
        // for raw in res {
        //     // let msg = JournalRecord::decode(raw.as_ref()).unwrap();
        //     println!("{:?}", raw);
        // }
    }

    #[tokio::test]
    async fn record_test() {
        let content = "c1".to_string().as_bytes().to_vec();
        let record = JournalRecord {
            content: content,
            create_time: now_second(),
            key: "k1".to_string(),
            namespace: "n1".to_string(),
            shard_name: "s1".to_string(),
            offset: 1000,
            segment: 1,
            tag: "t1".to_string(),
        };
        let mut msg = JournalRecord::encode_to_vec(&record);
        let br = br_recode_vec();
        msg.extend(br);
        assert!(JournalRecord::decode(msg.as_ref()).is_err());

        let end = msg.len() - 1;
        let resp = &msg[0..end];
        match JournalRecord::decode(resp.as_ref()) {
            Ok(data) => {
                assert_eq!(data, record);
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
