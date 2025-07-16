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

#![allow(clippy::result_large_err)]
use crate::{broker::MqttBroker, storage::message::build_message_storage_driver};
use common_base::version::logo::banner;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use handler::cache::CacheManager;
use std::sync::Arc;
use tokio::sync::broadcast;
pub mod admin;
pub mod bridge;
mod broker;
pub mod common;
pub mod handler;
pub mod observability;
pub mod security;
pub mod server;
pub mod storage;
mod subscribe;

pub fn start_broker(stop_send: broadcast::Sender<bool>) {
    banner();
    let conf = broker_config();
    let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(100));
    let metadata_cache: Arc<CacheManager> = Arc::new(CacheManager::new(
        client_pool.clone(),
        conf.cluster_name.clone(),
    ));

    let storage_driver = match build_message_storage_driver() {
        Ok(storage) => storage,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    let server = MqttBroker::new(
        client_pool,
        Arc::new(storage_driver),
        metadata_cache,
        stop_send.clone(),
    );
    server.start(stop_send);
}
