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

use common_config::broker::broker_config;
use storage_adapter::storage::ArcStorageAdapter;
use crate::{
    common::error::Result,
    storage::message::{Message}
};

#[derive(Clone, Default)]
pub struct Writer {
    storage_adapter: ArcStorageAdapter
}

pub fn cluster_name() -> String {
    let conf = broker_config();
    conf.cluster_name.clone()
}

fn shard_name(topic: String, partition: i32) -> String {
    format!("{}-{}", topic, partition)
}

impl Writer {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        Writer {storage_adapter}
    }

    pub async fn write(
        &self,
        msg : &Message,
    ) -> Result<()> {
        let namespace = cluster_name();
        let shard_name = shard_name(msg.topic_partition.topic, msg.topic_partition.partition);
        let _ = self
            .storage_adapter
            .batch_write(&namespace, &shard_name, &[msg.record])
            .await;
        Ok(())
    }

}