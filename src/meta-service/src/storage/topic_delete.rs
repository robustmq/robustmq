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

use crate::core::error::MetaServiceError;
use metadata_struct::topic::Topic;
use rocksdb_engine::keys::meta::{
    storage_key_cluster_delete_topic, storage_key_cluster_delete_topic_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_prefix_list_by_meta_metadata,
    engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct TopicDeleteStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TopicDeleteStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TopicDeleteStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, topic: &Topic) -> Result<(), MetaServiceError> {
        let key = storage_key_cluster_delete_topic(&topic.topic_id);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, topic)?;
        Ok(())
    }

    pub fn all(&self) -> Result<Vec<Topic>, MetaServiceError> {
        let prefix = storage_key_cluster_delete_topic_prefix();
        let data =
            engine_prefix_list_by_meta_metadata::<Topic>(&self.rocksdb_engine_handler, &prefix)?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, topic_id: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_cluster_delete_topic(topic_id);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }
}
