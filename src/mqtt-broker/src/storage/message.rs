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

use common_base::error::common::CommonError;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;
use storage_adapter::storage::ArcStorageAdapter;

#[derive(Clone)]
pub struct MessageStorage {
    storage_adapter: ArcStorageAdapter,
}

impl MessageStorage {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        MessageStorage { storage_adapter }
    }

    pub async fn append_topic_message(
        &self,
        topic_name: &str,
        record: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let shard_name = topic_name;
        let results = self
            .storage_adapter
            .batch_write(shard_name, &record)
            .await?;
        Ok(results)
    }

    pub async fn read_topic_message(
        &self,
        topic_name: &str,
        offset: u64,
        record_num: u64,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_name = topic_name;

        let mut read_config = ReadConfig::new();
        read_config.max_record_num = record_num;

        let records = self
            .storage_adapter
            .read_by_offset(shard_name, offset, &read_config)
            .await?;
        for raw in records.iter() {
            if !raw.crc32_check() {
                return Err(CommonError::CrcCheckByMessage);
            }
        }
        Ok(records)
    }

    pub async fn get_group_offset(
        &self,
        group_id: &str,
        shard_name: &str,
    ) -> Result<u64, CommonError> {
        for row in self
            .storage_adapter
            .get_offset_by_group(group_id)
            .await?
            .iter()
        {
            if *shard_name == row.shard_name {
                return Ok(row.offset);
            }
        }
        Ok(0)
    }

    pub async fn commit_group_offset(
        &self,
        group_id: &str,
        topic_name: &str,
        offset: u64,
    ) -> Result<(), CommonError> {
        let shard_name = topic_name;
        let mut offset_data = HashMap::new();
        offset_data.insert(shard_name.to_owned(), offset);

        self.storage_adapter
            .commit_offset(group_id, &offset_data)
            .await
    }
}
