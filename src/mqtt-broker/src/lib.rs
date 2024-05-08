use clients::poll::ClientPool;
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
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info,
    runtime::create_runtime,
};
use core::metadata_cache::{load_metadata_cache, MetadataCacheManager};
use core::{
    client_heartbeat::HeartbeatManager,
    keep_alive::KeepAlive,
    server_heartbeat::{register_broker_node, report_heartbeat, unregister_broker_node},
    session_expiry::SessionExpiry,
    HEART_CONNECT_SHARD_HASH_NUM,
};
use idempotent::memory::IdempotentMemory;
use server::{
    grpc::server::GrpcServer,
    http::server::{start_http_server, HttpServerState},
    start_mqtt_server,
    tcp::packet::{RequestPackage, ResponsePackage},
};
use std::sync::Arc;
use storage_adapter::{
    // memory::MemoryStorageAdapter,
    mysql::{build_mysql_conn_pool, MySQLStorageAdapter},
    // placement::PlacementStorageAdapter,
    storage::StorageAdapter,
};
use subscribe::{exclusive_sub::SubscribeExclusive, manager::SubscribeManager};
use tokio::{
    runtime::Runtime,
    signal,
    sync::broadcast::{self, Sender},
};

mod core;
mod handler;
mod idempotent;
mod metadata;
mod metrics;
mod security;
mod server;
mod storage;
mod subscribe;

pub fn start_mqtt_broker_server(stop_send: broadcast::Sender<bool>) {
    let conf = broker_mqtt_conf();
    let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(10));

    // let metadata_storage_adapter = Arc::new(PlacementStorageAdapter::new(
    //     client_poll.clone(),
    //     conf.placement_center.clone(),
    // ));

    // let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
    let pool = build_mysql_conn_pool(&conf.mysql.server).unwrap();
    let metadata_storage_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()));
    let message_storage_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()));
    let server = MqttBroker::new(
        client_poll,
        metadata_storage_adapter,
        message_storage_adapter,
    );
    server.start(stop_send)
}

pub struct MqttBroker<'a, T, S> {
    conf: &'a BrokerMQTTConfig,
    metadata_cache_manager: Arc<MetadataCacheManager>,
    heartbeat_manager: Arc<HeartbeatManager>,
    idempotent_manager: Arc<IdempotentMemory>,
    runtime: Runtime,
    request_queue_sx4: Sender<RequestPackage>,
    request_queue_sx5: Sender<RequestPackage>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    client_poll: Arc<ClientPool>,
    metadata_storage_adapter: Arc<T>,
    message_storage_adapter: Arc<S>,
    subscribe_manager: Arc<SubscribeManager>,
}

impl<'a, T, S> MqttBroker<'a, T, S>
where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        client_poll: Arc<ClientPool>,
        metadata_storage_adapter: Arc<T>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let runtime = create_runtime("storage-engine-server-runtime", conf.runtime.worker_threads);

        let (request_queue_sx4, _) = broadcast::channel(1000);
        let (request_queue_sx5, _) = broadcast::channel(1000);
        let (response_queue_sx4, _) = broadcast::channel(1000);
        let (response_queue_sx5, _) = broadcast::channel(1000);

        let heartbeat_manager = Arc::new(HeartbeatManager::new(HEART_CONNECT_SHARD_HASH_NUM));

        let metadata_cache = Arc::new(MetadataCacheManager::new("test-cluster".to_string()));
        let idempotent_manager: Arc<IdempotentMemory> = Arc::new(IdempotentMemory::new());
        let subscribe_manager = Arc::new(SubscribeManager::new(
            metadata_cache.clone(),
            client_poll.clone(),
        ));

        return MqttBroker {
            conf,
            runtime,
            metadata_cache_manager: metadata_cache,
            heartbeat_manager,
            idempotent_manager,
            request_queue_sx4,
            request_queue_sx5,
            response_queue_sx4,
            response_queue_sx5,
            client_poll,
            metadata_storage_adapter,
            message_storage_adapter,
            subscribe_manager,
        };
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.register_node();
        self.start_grpc_server();
        self.start_mqtt_server();
        self.start_http_server();
        self.start_keep_alive_thread(stop_send.subscribe());
        self.start_session_expiry_thread(stop_send.subscribe());
        self.start_cluster_heartbeat_report(stop_send.subscribe());
        self.start_push_server(stop_send.subscribe());
        self.awaiting_stop(stop_send);
    }

    fn start_mqtt_server(&self) {
        let cache = self.metadata_cache_manager.clone();
        let heartbeat_manager = self.heartbeat_manager.clone();
        let metadata_storage_adapter = self.metadata_storage_adapter.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let idempotent_manager = self.idempotent_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();

        let request_queue_sx4 = self.request_queue_sx4.clone();
        let request_queue_sx5 = self.request_queue_sx5.clone();

        let response_queue_sx4 = self.response_queue_sx4.clone();
        let response_queue_sx5 = self.response_queue_sx5.clone();
        self.runtime.spawn(async move {
            start_mqtt_server(
                subscribe_manager,
                cache,
                heartbeat_manager,
                metadata_storage_adapter,
                message_storage_adapter,
                idempotent_manager,
                request_queue_sx4,
                request_queue_sx5,
                response_queue_sx4,
                response_queue_sx5,
            )
            .await
        });
    }

    fn start_grpc_server(&self) {
        let server = GrpcServer::new(
            self.conf.grpc_port.clone(),
            self.metadata_cache_manager.clone(),
            self.metadata_storage_adapter.clone(),
        );
        self.runtime.spawn(async move {
            server.start().await;
        });
    }

    fn start_http_server(&self) {
        let http_state = HttpServerState::new(
            self.metadata_cache_manager.clone(),
            self.heartbeat_manager.clone(),
            self.response_queue_sx4.clone(),
            self.response_queue_sx5.clone(),
        );
        self.runtime
            .spawn(async move { start_http_server(http_state).await });
    }

    fn start_cluster_heartbeat_report(&self, stop_send: broadcast::Receiver<bool>) {
        let client_poll = self.client_poll.clone();
        self.runtime
            .spawn(async move { report_heartbeat(client_poll, stop_send).await });
    }

    fn start_push_server(&self, stop_send: broadcast::Receiver<bool>) {
        let subscribe_manager = self.subscribe_manager.clone();
        self.runtime.spawn(async move {
            subscribe_manager.start().await;
        });

        let exclusive_sub = SubscribeExclusive::new(
            self.message_storage_adapter.clone(),
            self.metadata_cache_manager.clone(),
            self.response_queue_sx4.clone(),
            self.response_queue_sx5.clone(),
            self.subscribe_manager.clone(),
        );

        self.runtime.spawn(async move {
            exclusive_sub.start().await;
        });
    }

    fn start_keep_alive_thread(&self, stop_send: broadcast::Receiver<bool>) {
        let mut keep_alive = KeepAlive::new(
            HEART_CONNECT_SHARD_HASH_NUM,
            self.heartbeat_manager.clone(),
            self.request_queue_sx4.clone(),
            self.request_queue_sx5.clone(),
            stop_send,
        );
        self.runtime.spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });
    }

    fn start_session_expiry_thread(&self, stop_send: broadcast::Receiver<bool>) {
        let sesssion_expiry = SessionExpiry::new();
        self.runtime.spawn(async move {
            sesssion_expiry.start_session_expire_check().await;
        });
    }

    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        // Wait for the stop signal
        self.runtime.block_on(async move {
            loop {
                signal::ctrl_c().await.expect("failed to listen for event");
                match stop_send.send(true) {
                    Ok(_) => {
                        info("When ctrl + c is received, the service starts to stop".to_string());
                        self.stop_server().await;
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        // todo tokio runtime shutdown
    }

    fn register_node(&self) {
        let metadata_cache = self.metadata_cache_manager.clone();
        let metadata_storage_adapter = self.metadata_storage_adapter.clone();
        self.runtime.block_on(async move {
            // metadata_cache.init_metadata_data(load_metadata_cache(metadata_storage_adapter).await);
            register_broker_node(self.client_poll.clone()).await;
        });
    }

    async fn stop_server(&self) {
        // unregister node
        unregister_broker_node(self.client_poll.clone()).await;
    }
}
