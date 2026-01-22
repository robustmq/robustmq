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

mod connect;
mod disconnect;
mod ping;
mod publish;
mod qos_ack;
mod subscribe;

use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{
    Connect, ConnectProperties, LastWill, LastWillProperties, Login, MqttProtocol,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use schema_register::schema::SchemaRegisterManager;
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::core::cache::MQTTCacheManager;
use crate::security::AuthDriver;
use crate::subscribe::manager::SubscribeManager;

use delay_message::manager::DelayMessageManager;

#[derive(Clone)]
pub struct MqttService {
    protocol: MqttProtocol,
    cache_manager: Arc<MQTTCacheManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    delay_message_manager: Arc<DelayMessageManager>,
    subscribe_manager: Arc<SubscribeManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    client_pool: Arc<ClientPool>,
    auth_driver: Arc<AuthDriver>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

#[derive(Clone)]
pub struct MqttServiceContext {
    pub protocol: MqttProtocol,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub client_pool: Arc<ClientPool>,
    pub auth_driver: Arc<AuthDriver>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

#[derive(Clone)]
pub struct MqttServiceConnectContext {
    pub connect_id: u64,
    pub connect: Connect,
    pub connect_properties: Option<ConnectProperties>,
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
    pub login: Option<Login>,
    pub addr: SocketAddr,
}

impl MqttService {
    pub fn new(context: MqttServiceContext) -> Self {
        MqttService {
            protocol: context.protocol,
            cache_manager: context.cache_manager,
            connection_manager: context.connection_manager,
            storage_driver_manager: context.storage_driver_manager,
            delay_message_manager: context.delay_message_manager,
            subscribe_manager: context.subscribe_manager,
            client_pool: context.client_pool,
            auth_driver: context.auth_driver,
            schema_manager: context.schema_manager,
            rocksdb_engine_handler: context.rocksdb_engine_handler,
        }
    }
}
