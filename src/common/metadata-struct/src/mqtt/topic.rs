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

use common_base::{
    error::common::CommonError,
    tools::{now_second, unique_id},
    utils::serialize,
};
use common_config::storage::StorageType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Topic {
    pub topic_id: String,
    pub topic_name: String,
    pub storage_type: StorageType,
    pub partition: u32,
    pub replication: u32,
    pub storage_name_list: Vec<String>,
    pub create_time: u64,
}

impl Topic {
    pub fn build_by_name(topic_name: &str) -> Self {
        let unique_id = unique_id();
        Topic {
            topic_id: unique_id.clone(),
            topic_name: topic_name.to_string(),
            storage_type: StorageType::EngineRocksDB,
            partition: 1,
            replication: 1,
            storage_name_list: Topic::create_partition_name(&unique_id, 1),
            create_time: now_second(),
        }
    }

    pub fn create_partition_name(topic_id: &str, partition: u32) -> Vec<String> {
        let mut results = Vec::new();
        for i in 0..partition {
            results.push(Topic::build_storage_name(topic_id, i));
        }
        results
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }

    pub fn build_storage_name(topic_id: &str, partition: u32) -> String {
        format!("{}-{}", topic_id, partition)
    }
}
