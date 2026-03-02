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
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, storage_record::StorageRecord,
};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::driver::StorageDriverManager;

#[derive(Clone)]
pub struct MessageStorage {
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

impl MessageStorage {
    pub fn new(storage_driver_manager: Arc<StorageDriverManager>) -> Self {
        MessageStorage {
            storage_driver_manager,
        }
    }

    pub async fn append_topic_message(
        &self,
        topic_name: &str,
        records: Vec<AdapterWriteRecord>,
    ) -> Result<Vec<u64>, CommonError> {
        let results = self
            .storage_driver_manager
            .write(topic_name, &records)
            .await?;
        let mut offsets = Vec::new();
        for row in results {
            if row.is_error() {
                return Err(CommonError::CommonError(row.error_info()));
            }
            offsets.push(row.offset);
        }
        Ok(offsets)
    }

    pub async fn read_topic_message(
        &self,
        topic_name: &str,
        offsets: &HashMap<String, u64>,
        max_record_num: u64,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let read_config = AdapterReadConfig {
            max_record_num,
            max_size: 1024 * 1024 * 30,
        };

        self.storage_driver_manager
            .read_by_offset(topic_name, offsets, &read_config)
            .await
    }

    pub async fn get_group_offset(
        &self,
        group_id: &str,
    ) -> Result<HashMap<String, u64>, CommonError> {
        let resp = self
            .storage_driver_manager
            .offset_manager
            .get_offset(group_id)
            .await?;
        let mut results = HashMap::with_capacity(2);
        for raw in resp {
            results.insert(raw.shard_name, raw.offset);
        }
        Ok(results)
    }

    pub async fn commit_group_offset(
        &self,
        group_id: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.storage_driver_manager
            .offset_manager
            .commit_offset(group_id, offsets)
            .await
    }
}
