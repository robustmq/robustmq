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

use cache::{load_node_cache, start_update_cache_thread, MetadataCache};
use connection::{start_conn_gc_thread, ConnectionManager};
use error::JournalClientError;
use option::JournalClientOption;
use tokio::sync::broadcast::{self, Sender};

pub mod admin;
mod async_reader;
mod async_writer;
mod cache;
mod connection;
mod error;
pub mod option;
pub mod read;
mod service;
pub mod tool;
pub mod write;

#[derive(Clone)]
pub struct JournalEngineClient {
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    stop_send: Sender<bool>,
}

impl JournalEngineClient {
    pub fn new(options: JournalClientOption) -> Self {
        let metadata_cache = Arc::new(MetadataCache::new(options.addrs));
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));
        let (stop_send, _) = broadcast::channel::<bool>(2);
        JournalEngineClient {
            metadata_cache,
            connection_manager,
            stop_send,
        }
    }

    pub async fn connect(&self) -> Result<(), JournalClientError> {
        load_node_cache(&self.metadata_cache, &self.connection_manager).await?;

        start_update_cache_thread(
            self.metadata_cache.clone(),
            self.connection_manager.clone(),
            self.stop_send.subscribe(),
        );

        start_conn_gc_thread(
            self.metadata_cache.clone(),
            self.connection_manager.clone(),
            self.stop_send.subscribe(),
        );
        Ok(())
    }
}
