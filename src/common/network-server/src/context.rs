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

use crate::{command::ArcCommandAdapter, common::connection_manager::ConnectionManager};
use broker_core::cache::BrokerCacheManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnectionType;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy)]
pub struct ProcessorConfig {
    pub accept_thread_num: usize,
    pub handler_process_num: usize,
    pub response_process_num: usize,
    pub channel_size: usize,
}

#[derive(Clone)]
pub struct ServerContext {
    pub connection_manager: Arc<ConnectionManager>,
    pub client_pool: Arc<ClientPool>,
    pub command: ArcCommandAdapter,
    pub network_type: NetworkConnectionType,
    pub proc_config: ProcessorConfig,
    pub broker_cache: Arc<BrokerCacheManager>,
    pub stop_sx: broadcast::Sender<bool>,
}
