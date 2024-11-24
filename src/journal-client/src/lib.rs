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

#![allow(dead_code, unused_variables)]

use std::sync::Arc;

use cache::{load_node_cache, MetadataCache};
use connection::ConnectionManager;
use error::JournalClientError;
use option::JournalClientOption;

mod cache;
mod connection;
mod connection_gc;
mod error;
pub mod option;
mod service;
pub mod tool;

pub struct JournalEngineClient {
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
}

impl JournalEngineClient {
    pub fn new(options: JournalClientOption) -> Self {
        let metadata_cache = Arc::new(MetadataCache::new(options.addrs));
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));
        JournalEngineClient {
            metadata_cache,
            connection_manager,
        }
    }

    pub async fn connect(&self) -> Result<(), JournalClientError> {
        load_node_cache(&self.metadata_cache, &self.connection_manager).await?;
        Ok(())
    }

    pub async fn create_shard(&self) {}

    pub fn delete_shard(&self) {}

    pub fn send(&self) {}

    pub fn read(&self) {}

    pub fn close(&self) {}
}
