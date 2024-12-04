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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use metadata_struct::adapter::record::Record;
use storage_adapter::storage::StorageAdapter;

pub fn cluster_name() -> String {
    let conf = broker_mqtt_conf();
    conf.cluster_name.clone()
}

#[derive(Clone)]
pub struct MessageStorage<T> {
    storage_adapter: Arc<T>,
}

impl<T> MessageStorage<T>
where
    T: StorageAdapter + Send + Sync + 'static,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
        MessageStorage { storage_adapter }
    }

    // Save the data for the Topic dimension
    pub async fn append_topic_message(
        &self,
        topic_id: String,
        record: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError> {
        let shard_name = topic_id;
        let namespace = cluster_name();
        match self
            .storage_adapter
            .stream_write(namespace, shard_name, record)
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(e),
        }
    }

    // Read the data for the Topic dimension
    pub async fn read_topic_message(
        &self,
        topic_id: String,
        group_id: String,
        record_num: u128,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_name = topic_id;
        let namespace = cluster_name();
        match self
            .storage_adapter
            .stream_read(namespace, shard_name, group_id, Some(record_num), None)
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    // Submits the offset information for consumption
    pub async fn commit_group_offset(
        &self,
        topic_id: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, CommonError> {
        let shard_name = topic_id;
        let namespace = cluster_name();
        match self
            .storage_adapter
            .stream_commit_offset(namespace, shard_name, group_id, offset)
            .await
        {
            Ok(flag) => Ok(flag),
            Err(e) => Err(e),
        }
    }
}
