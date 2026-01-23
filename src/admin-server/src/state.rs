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

use broker_core::cache::BrokerCacheManager;
use grpc_clients::pool::ClientPool;
use mqtt_broker::{
    bridge::manager::ConnectorManager, core::cache::MQTTCacheManager,
    subscribe::manager::SubscribeManager,
};
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::RateLimiterManager;
use rocksdb_engine::{metrics::mqtt::MQTTMetricsCache, rocksdb::RocksDBEngine};
use schema_register::schema::SchemaRegisterManager;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::{core::cache::StorageCacheManager, handler::adapter::StorageEngineHandler};

#[derive(Clone)]
pub struct HttpState {
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<BrokerCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub mqtt_context: MQTTContext,
    pub engine_context: StorageEngineContext,
    pub rate_limiter_manager: Arc<RateLimiterManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

#[derive(Clone)]
pub struct MQTTContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub metrics_manager: Arc<MQTTMetricsCache>,
    pub connector_manager: Arc<ConnectorManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
}

#[derive(Clone)]
pub struct StorageEngineContext {
    pub cache_manager: Arc<StorageCacheManager>,
    pub engine_adapter_handler: Arc<StorageEngineHandler>,
}
