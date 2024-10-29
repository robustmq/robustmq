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

use common_base::tools::file_exists;
use tokio::fs::{create_dir, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::core::error::JournalServerError;

#[derive(Default)]
pub struct SegmentFile {
    pub namespace: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub file_path: String,
    pub is_open: bool,
}

impl SegmentFile {
    pub fn new(namespace: String, shard_name: String, segment_no: u32, data_fold: String) -> Self {
        let mut segment = SegmentFile {
            namespace,
            shard_name,
            segment_no,
            is_open: false,
            ..Default::default()
        };
        segment.file_path = segment.build_file_path(
            &segment.namespace,
            &segment.shard_name,
            &data_fold,
            segment_no,
        );
        segment
    }

    async fn create(&self) -> Result<(), JournalServerError> {
        if file_exists(&self.file_path) {
            return Err(JournalServerError::SegmentFileAlreadyExists(
                self.file_path.clone(),
            ));
        }
        File::create(self.file_path.clone()).await?;
        Ok(())
    }

    pub async fn write(&self, data: Vec<u8>) -> Result<(), JournalServerError> {
        let file = self.open().await?;
        let mut writer = tokio::io::BufWriter::new(file);

        writer.write_all(data.as_slice()).await?;
        Ok(())
    }

    pub async fn read(
        &self,
        offset: Option<u64>,
        size: u64,
    ) -> Result<Vec<Vec<u8>>, JournalServerError> {
        let file = self.open().await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut results = Vec::new();
        let already_read_size = 0;
        loop {
            if already_read_size >= size {
                break;
            }
            let mut data = Vec::new();
            reader.read_until(b'\n', &mut data).await?;
            results.push(data);
        }
        Ok(results)
    }

    async fn open(&self) -> Result<File, JournalServerError> {
        if file_exists(&self.file_path) {
            return Err(JournalServerError::SegmentFileNotExists(
                self.file_path.clone(),
            ));
        }
        Ok(File::open(self.file_path.clone()).await?)
    }

    fn build_file_path(
        &self,
        namespace: &str,
        shard_name: &str,
        data_fold: &str,
        segment_no: u32,
    ) -> String {
        let file_name = format!("{}/{}-{}", namespace, shard_name, segment_no);
        format!("{}/{}", data_fold, file_name)
    }

    fn seek_offset(&self, offfset: u64) -> Result<u64, JournalServerError> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::SegmentFile;

    #[tokio::test]
    async fn segment_create() {
        let namespace = "n1";
        let shard_name = "s1";
        let segment_no = 10;
        let data_fold = "/tmp/tests/jl";
        let segment = SegmentFile::new(
            namespace.to_string(),
            shard_name.to_string(),
            segment_no,
            data_fold.to_string(),
        );
        match segment.create().await {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
