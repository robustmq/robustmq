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

use broker_core::cache::NodeCacheManager;
use common_security::manager::SecurityManager;
use connector::manager::ConnectorManager;
use grpc_clients::pool::ClientPool;
use mqtt_broker::{
    core::{cache::MQTTCacheManager, retain::RetainMessageManager},
    subscribe::{manager::SubscribeManager, PushManager},
};
use nats_broker::core::cache::NatsCacheManager;
use nats_broker::push::manager::NatsSubscribeManager;
use network_server::common::connection_manager::ConnectionManager;
use rate_limit::global::GlobalRateLimiterManager;
use rocksdb_engine::{metrics::mqtt::MQTTMetricsCache, rocksdb::RocksDBEngine};
use schema_register::schema::SchemaRegisterManager;
use storage_adapter::driver::StorageDriverManager;
use storage_engine::{core::cache::StorageCacheManager, handler::adapter::StorageEngineHandler};

#[derive(Clone)]
pub struct HttpState {
    pub client_pool: Arc<ClientPool>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub mqtt_context: MQTTContext,
    pub engine_context: StorageEngineContext,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub rate_limiter: Arc<GlobalRateLimiterManager>,
    /// Present only when the nats-broker is co-started with admin-server.
    pub nats_context: Option<NatsContext>,
}

/// Context carrying nats-broker runtime state.
/// Injected into HttpState only when the nats-broker is running alongside admin-server.
#[derive(Clone)]
pub struct NatsContext {
    pub cache_manager: Arc<NatsCacheManager>,
    pub subscribe_manager: Arc<NatsSubscribeManager>,
}

#[derive(Clone)]
pub struct MQTTContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub security_manager: Arc<SecurityManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub metrics_manager: Arc<MQTTMetricsCache>,
    pub connector_manager: Arc<ConnectorManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub retain_message_manager: Arc<RetainMessageManager>,
    pub push_manager: Arc<PushManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
}

#[derive(Clone)]
pub struct StorageEngineContext {
    pub cache_manager: Arc<StorageCacheManager>,
    pub engine_adapter_handler: Arc<StorageEngineHandler>,
}
