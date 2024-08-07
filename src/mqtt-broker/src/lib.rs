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
use clients::poll::ClientPool;
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info, runtime::create_runtime};
use handler::keep_alive::ClientKeepAlive;
use handler::{cache_manager::CacheManager, heartbreat::report_heartbeat};
use security::AuthDriver;
use server::connection_manager::ConnectionManager;
use server::tcp::start_tcp_server;
use server::websocket::server::{websocket_server, websockets_server, WebSocketServerState};
use server::{
    grpc::server::GrpcServer,
    http::server::{start_http_server, HttpServerState},
};
use std::sync::Arc;
use storage::cluster::ClusterStorage;
use storage_adapter::memory::MemoryStorageAdapter;
use storage_adapter::mysql::MySQLStorageAdapter;
use storage_adapter::storage::StorageAdapter;
use storage_adapter::{storage_is_memory, storage_is_mysql};
use subscribe::{
    sub_exclusive::SubscribeExclusive, sub_share_follower::SubscribeShareFollower,
    sub_share_leader::SubscribeShareLeader, subscribe_manager::SubscribeManager,
};
use third_driver::mysql::build_mysql_conn_pool;
use tokio::{
    runtime::Runtime,
    signal,
    sync::broadcast::{self},
};

mod handler;
mod metrics;
mod security;
mod server;
pub mod storage;
mod subscribe;

pub fn start_mqtt_broker_server(stop_send: broadcast::Sender<bool>) {
    let conf = broker_mqtt_conf();
    let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(100));
    let metadata_cache = Arc::new(CacheManager::new(
        client_poll.clone(),
        conf.cluster_name.clone(),
    ));
    let storage_type = conf.storage.storage_type.clone();
    if storage_is_memory(&storage_type) {
        let message_storage_adapter = Arc::new(MemoryStorageAdapter::new());
        let server = MqttBroker::new(client_poll, message_storage_adapter, metadata_cache);
        server.start(stop_send);
    } else if storage_is_mysql(&storage_type) {
        if conf.storage.mysql_addr.is_empty() {
            panic!("storaget type is [mysql],[storage.mysql_addr] cannot be empty");
        }
        let pool = build_mysql_conn_pool(&conf.storage.mysql_addr).unwrap();
        let message_storage_adapter = Arc::new(MySQLStorageAdapter::new(pool.clone()));
        let server = MqttBroker::new(client_poll, message_storage_adapter, metadata_cache);
        server.start(stop_send);
    } else {
        panic!("Message data storage type configuration error, optional :mysql, memory");
    };
}

pub struct MqttBroker<S> {
    cache_manager: Arc<CacheManager>,
    runtime: Runtime,
    client_poll: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    auth_driver: Arc<AuthDriver>,
}

impl<S> MqttBroker<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        let conf = broker_mqtt_conf();
        let runtime = create_runtime(
            "storage-engine-server-runtime",
            conf.system.runtime_worker_threads,
        );

        let subscribe_manager = Arc::new(SubscribeManager::new(
            cache_manager.clone(),
            client_poll.clone(),
        ));

        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));

        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_poll.clone()));
        return MqttBroker {
            runtime,
            cache_manager,
            client_poll,
            message_storage_adapter,
            subscribe_manager,
            connection_manager,
            auth_driver,
        };
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.register_node();
        self.start_grpc_server();
        self.start_mqtt_server(stop_send.clone());
        self.start_http_server();
        self.start_websocket_server(stop_send.clone());
        self.start_keep_alive_thread(stop_send.clone());
        self.start_cluster_heartbeat_report(stop_send.clone());
        self.start_push_server();
        self.awaiting_stop(stop_send);
    }

    fn start_mqtt_server(&self, stop_send: broadcast::Sender<bool>) {
        let cache = self.cache_manager.clone();
        let message_storage_adapter = self.message_storage_adapter.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let client_poll = self.client_poll.clone();
        let connection_manager = self.connection_manager.clone();
        let auth_driver = self.auth_driver.clone();

        self.runtime.spawn(async move {
            start_tcp_server(
                subscribe_manager,
                cache,
                connection_manager,
                message_storage_adapter,
                client_poll,
                stop_send,
                auth_driver,
            )
            .await
        });
    }

    fn start_grpc_server(&self) {
        let conf = broker_mqtt_conf();
        let server = GrpcServer::new(
            conf.grpc_port.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.client_poll.clone(),
            self.message_storage_adapter.clone(),
        );
        self.runtime.spawn(async move {
            server.start().await;
        });
    }

    fn start_http_server(&self) {
        let http_state =
            HttpServerState::new(self.cache_manager.clone(), self.subscribe_manager.clone());
        self.runtime
            .spawn(async move { start_http_server(http_state).await });
    }

    fn start_websocket_server(&self, stop_send: broadcast::Sender<bool>) {
        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );
        self.runtime
            .spawn(async move { websocket_server(ws_state).await });

        let ws_state = WebSocketServerState::new(
            self.subscribe_manager.clone(),
            self.cache_manager.clone(),
            self.connection_manager.clone(),
            self.message_storage_adapter.clone(),
            self.client_poll.clone(),
            self.auth_driver.clone(),
            stop_send.clone(),
        );

        self.runtime
            .spawn(async move { websockets_server(ws_state).await });
    }

    fn start_cluster_heartbeat_report(&self, stop_send: broadcast::Sender<bool>) {
        let client_poll = self.client_poll.clone();
        self.runtime.spawn(async move {
            report_heartbeat(client_poll, stop_send).await;
        });
    }

    fn start_push_server(&self) {
        let subscribe_manager = self.subscribe_manager.clone();
        self.runtime.spawn(async move {
            subscribe_manager.start().await;
        });

        let exclusive_sub = SubscribeExclusive::new(
            self.message_storage_adapter.clone(),
            self.cache_manager.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            exclusive_sub.start().await;
        });

        let leader_sub = SubscribeShareLeader::new(
            self.subscribe_manager.clone(),
            self.message_storage_adapter.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            leader_sub.start().await;
        });

        let follower_sub = SubscribeShareFollower::new(
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            self.client_poll.clone(),
        );

        self.runtime.spawn(async move {
            follower_sub.start().await;
        });
    }

    fn start_keep_alive_thread(&self, stop_send: broadcast::Sender<bool>) {
        let mut keep_alive = ClientKeepAlive::new(
            self.client_poll.clone(),
            self.subscribe_manager.clone(),
            self.connection_manager.clone(),
            self.cache_manager.clone(),
            stop_send,
        );
        self.runtime.spawn(async move {
            keep_alive.start_heartbeat_check().await;
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
        let metadata_cache = self.cache_manager.clone();
        let client_poll = self.client_poll.clone();
        self.runtime.block_on(async move {
            metadata_cache.init_system_user().await;
            metadata_cache.load_metadata_cache().await;

            let cluster_storage = ClusterStorage::new(client_poll.clone());
            cluster_storage.register_node().await;
        });
    }

    async fn stop_server(&self) {
        let cluster_storage = ClusterStorage::new(self.client_poll.clone());
        cluster_storage.unregister_node().await;
        self.connection_manager.close_all_connect().await;
    }
}
