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

use axum::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::adapter::record::Record;

#[derive(Default)]
pub struct ShardConfig {
    pub replica_num: u32,
}

#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), CommonError>;

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError>;

    // Streaming storage model: Append data in a Shard dimension, returning a unique self-incrementing ID for the Shard dimension
    async fn stream_write(
        &self,
        namespace: String,
        shard_name: String,
        data: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError>;

    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_read(
        &self,
        namespace: String,
        shard_name: String,
        group_id: String,
        record_num: Option<u128>,
        record_size: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError>;

    // Streaming storage model: Read the next batch of data in the dimension of the Shard + subscription name tuple
    async fn stream_commit_offset(
        &self,
        namespace: String,
        shard_name: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, CommonError>;

    // Streaming storage model: A piece of data is uniquely read based on the shard name and a unique auto-incrementing ID.
    async fn stream_read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        record_id: usize,
    ) -> Result<Option<Record>, CommonError>;

    // Streaming storage model: A batch of data is read based on the shard name and time range.
    async fn stream_read_by_timestamp(
        &self,
        namespace: String,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
        record_num: Option<usize>,
        record_size: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError>;

    // Streaming storage model: A batch of data is read based on the shard name and the last time it expires
    async fn stream_read_by_key(
        &self,
        namespace: String,
        shard_name: String,
        key: String,
    ) -> Result<Option<Record>, CommonError>;

    async fn close(&self) -> Result<(), CommonError>;
}
