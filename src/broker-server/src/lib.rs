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

use crate::{connection::network_connection_gc, grpc::start_grpc_server};
use admin_server::{
    server::AdminServer,
    state::{HttpState, MQTTContext},
};
use broker_core::{
    cache::BrokerCacheManager,
    heartbeat::{check_meta_service_status, register_node, report_heartbeat},
};
use common_base::{
    role::{is_broker_node, is_engine_node, is_meta_node},
    runtime::create_runtime,
};
use common_config::{broker::broker_config, config::BrokerConfig};
use common_metrics::core::server::register_prometheus_export;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use meta_service::{
    controller::call_broker::call::BrokerCallManager,
    core::cache::CacheManager as PlacementCacheManager,
    raft::{manager::MultiRaftManager, route::DataRoute},
    MetaServiceServer, MetaServiceServerParams,
};
use metadata_struct::adapter::MessageExpireConfig;
use mqtt_broker::{
    bridge::manager::ConnectorManager,
    broker::{MqttBrokerServer, MqttBrokerServerParams},
    handler::cache::MQTTCacheManager as MqttCacheManager,
    security::AuthDriver,
    subscribe::manager::SubscribeManager,
};
use network_server::common::connection_manager::ConnectionManager as NetworkConnectionManager;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use rate_limit::RateLimiterManager;
use rocksdb_engine::{
    metrics::mqtt::MQTTMetricsCache,
    rocksdb::RocksDBEngine,
    storage::family::{column_family_list, storage_data_fold},
};
use schema_register::schema::SchemaRegisterManager;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use storage_adapter::{
    driver::build_message_storage_driver, expire::message_expire_thread, offset::OffsetManager,
    storage::ArcStorageAdapter,
};
use storage_engine::{
    core::cache::StorageCacheManager, segment::write::WriteManager, StorageEngineParams,
    StorageEngineServer,
};
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tracing::{error, info};

mod cluster_service;
pub mod common;
mod connection;
mod grpc;

pub struct BrokerServer {
    main_runtime: Runtime,
    place_params: MetaServiceServerParams,
    mqtt_params: MqttBrokerServerParams,
    journal_params: StorageEngineParams,
    client_pool: Arc<ClientPool>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rate_limiter_manager: Arc<RateLimiterManager>,
    connection_manager: Arc<NetworkConnectionManager>,
    broker_cache: Arc<BrokerCacheManager>,
    offset_manager: Arc<OffsetManager>,
    config: BrokerConfig,
}

impl Default for BrokerServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BrokerServer {
    pub fn new() -> Self {
        let config = broker_config();
        let client_pool = Arc::new(ClientPool::new(100));
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &storage_data_fold(&config.rocksdb.data_path),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let rate_limiter_manager = Arc::new(RateLimiterManager::new());
        let main_runtime = create_runtime("init_runtime", config.runtime.runtime_worker_threads);
        let broker_cache = Arc::new(BrokerCacheManager::new(config.clone()));
        let connection_manager = Arc::new(NetworkConnectionManager::new(
            config.network.lock_max_try_mut_times as i32,
            config.network.lock_try_mut_sleep_time_ms,
        ));
        let place_params = main_runtime.block_on(async {
            BrokerServer::build_meta_service(
                client_pool.clone(),
                rocksdb_engine_handler.clone(),
                broker_cache.clone(),
            )
            .await
        });

        let offset_manager = Arc::new(OffsetManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
        ));

        let raw_offset_manager = offset_manager.clone();
        let raw_rocksdb_engine_handler = rocksdb_engine_handler.clone();
        let message_storage_adapter = main_runtime.block_on(async move {
            let storage = match build_message_storage_driver(
                raw_offset_manager.clone(),
                raw_rocksdb_engine_handler.clone(),
                config.message_storage.clone(),
            )
            .await
            {
                Ok(storage) => storage,
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            };
            storage
        });

        let mqtt_params = BrokerServer::build_broker_mqtt_params(
            client_pool.clone(),
            broker_cache.clone(),
            rocksdb_engine_handler.clone(),
            connection_manager.clone(),
            message_storage_adapter,
            offset_manager.clone(),
        );

        let journal_params = BrokerServer::build_storage_engine_params(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            broker_cache.clone(),
            connection_manager.clone(),
        );

        BrokerServer {
            broker_cache,
            main_runtime,
            place_params,
            journal_params,
            config: config.clone(),
            mqtt_params,
            client_pool,
            rocksdb_engine_handler,
            rate_limiter_manager,
            connection_manager,
            offset_manager,
        }
    }

    pub fn start(&self) {
        // start grpc server
        let place_params = self.place_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let journal_params = self.journal_params.clone();
        let broker_cache = self.broker_cache.clone();
        let server_runtime =
            create_runtime("server-runtime", self.config.runtime.runtime_worker_threads);
        let grpc_port = self.config.grpc_port;

        let grpc_ready = Arc::new(AtomicBool::new(false));
        let grpc_ready_check = grpc_ready.clone();
        server_runtime.spawn(async move {
            if let Err(e) =
                start_grpc_server(place_params, mqtt_params, journal_params, grpc_port).await
            {
                panic!("{e}")
            }
        });

        // Start Admin Server
        let state = Arc::new(HttpState {
            client_pool: self.client_pool.clone(),
            connection_manager: self.mqtt_params.connection_manager.clone(),
            mqtt_context: MQTTContext {
                cache_manager: self.mqtt_params.cache_manager.clone(),
                subscribe_manager: self.mqtt_params.subscribe_manager.clone(),
                metrics_manager: self.mqtt_params.metrics_cache_manager.clone(),
                connector_manager: self.mqtt_params.connector_manager.clone(),
                schema_manager: self.mqtt_params.schema_manager.clone(),
            },
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            broker_cache,
            rate_limiter_manager: self.rate_limiter_manager.clone(),
            storage_adapter: self.mqtt_params.message_storage_adapter.clone(),
        });

        let http_port = self.config.http_port;
        server_runtime.spawn(async move {
            let admin_server = AdminServer::new();
            admin_server.start(http_port, state).await;
        });

        // check grpc server ready
        self.check_grpc_server_ready(grpc_port, grpc_ready_check);

        // start pprof server
        server_runtime.spawn(async move {
            let conf = broker_config();
            if conf.p_prof.enable {
                start_pprof_monitor(conf.p_prof.port, conf.p_prof.frequency).await;
            }
        });

        // start prometheus
        let prometheus_port = self.config.prometheus.port;
        if self.config.prometheus.enable {
            server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }

        self.wait_for_grpc_ready(&grpc_ready);

        let mut place_stop_send = None;
        let mut mqtt_stop_send = None;
        let mut journal_stop_send = None;

        let config = broker_config();
        // start meta service
        let (stop_send, _) = broadcast::channel(2);
        let place_runtime =
            create_runtime("place-runtime", self.config.runtime.runtime_worker_threads);
        let place_params = self.place_params.clone();
        if is_meta_node(&config.roles) {
            place_stop_send = Some(stop_send.clone());
            place_runtime.spawn(async move {
                let mut pc = MetaServiceServer::new(place_params, stop_send.clone());
                pc.start().await;
            });

            // check placement ready
            self.main_runtime.block_on(async {
                check_meta_service_status(self.client_pool.clone()).await;
            });
        }

        // start journal server
        let (stop_send, _) = broadcast::channel(2);
        let journal_runtime = create_runtime(
            "journal-runtime",
            self.config.runtime.runtime_worker_threads,
        );

        if is_engine_node(&config.roles) {
            journal_stop_send = Some(stop_send.clone());
            let server = StorageEngineServer::new(self.journal_params.clone(), stop_send);
            journal_runtime.spawn(async move {
                server.start().await;
            });
            self.wait_for_journal_ready();
        }

        // start mqtt server
        let (stop_send, _) = broadcast::channel(2);
        let mqtt_runtime =
            create_runtime("mqtt-runtime", self.config.runtime.runtime_worker_threads);
        if is_broker_node(&config.roles) {
            mqtt_stop_send = Some(stop_send.clone());
            let server = MqttBrokerServer::new(self.mqtt_params.clone(), stop_send.clone());
            mqtt_runtime.spawn(async move {
                server.start().await;
            });
        }

        // register node
        let raw_stop_send = stop_send.clone();
        server_runtime.block_on(async move {
            self.register_node(raw_stop_send.clone()).await;
        });

        // connection gc
        let connection_manager = self.connection_manager.clone();
        let raw_stop_send = stop_send.clone();
        server_runtime
            .spawn(async move { network_connection_gc(connection_manager, raw_stop_send).await });

        // offset flush thread
        let offset_cache = self.offset_manager.clone();
        let raw_stop_send = stop_send;
        server_runtime.spawn(async move {
            offset_cache.offset_save_thread(raw_stop_send).await;
        });

        // message expire
        let storage = self.mqtt_params.message_storage_adapter.clone();
        server_runtime.spawn(async move {
            message_expire_thread(storage.clone(), MessageExpireConfig::default()).await;
        });

        // awaiting stop
        self.awaiting_stop(place_stop_send, mqtt_stop_send, journal_stop_send);
    }

    async fn build_meta_service(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        broker_cache: Arc<BrokerCacheManager>,
    ) -> MetaServiceServerParams {
        let cache_manager = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let call_manager = Arc::new(BrokerCallManager::new(broker_cache));

        let data_route = Arc::new(DataRoute::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
        ));
        let raft_manager = Arc::new(
            match MultiRaftManager::new(
                client_pool.clone(),
                rocksdb_engine_handler.clone(),
                data_route,
            )
            .await
            {
                Ok(data) => data,
                Err(e) => {
                    panic!("{}", e);
                }
            },
        );

        MetaServiceServerParams {
            cache_manager,
            rocksdb_engine_handler,
            client_pool,
            call_manager,
            raft_manager,
        }
    }

    fn build_broker_mqtt_params(
        client_pool: Arc<ClientPool>,
        broker_cache: Arc<BrokerCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        connection_manager: Arc<NetworkConnectionManager>,
        message_storage_adapter: ArcStorageAdapter,
        offset_manager: Arc<OffsetManager>,
    ) -> MqttBrokerServerParams {
        let cache_manager = Arc::new(MqttCacheManager::new(
            client_pool.clone(),
            broker_cache.clone(),
        ));
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connector_manager = Arc::new(ConnectorManager::new());
        let auth_driver = Arc::new(AuthDriver::new(cache_manager.clone(), client_pool.clone()));
        let delay_message_manager =
            Arc::new(DelayMessageManager::new(1, message_storage_adapter.clone()));
        let metrics_cache_manager = Arc::new(MQTTMetricsCache::new(rocksdb_engine_handler.clone()));
        let schema_manager = Arc::new(SchemaRegisterManager::new());

        MqttBrokerServerParams {
            cache_manager,
            client_pool,
            message_storage_adapter,
            subscribe_manager,
            connection_manager,
            connector_manager,
            auth_driver,
            delay_message_manager,
            schema_manager,
            metrics_cache_manager,
            rocksdb_engine_handler,
            broker_cache,
            offset_manager,
        }
    }

    fn build_storage_engine_params(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        broker_cache: Arc<BrokerCacheManager>,
        connection_manager: Arc<NetworkConnectionManager>,
    ) -> StorageEngineParams {
        let config = broker_config();
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache.clone()));

        let write_manager = Arc::new(WriteManager::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            client_pool.clone(),
            config.storage_runtime.io_thread_num,
        ));
        StorageEngineParams {
            cache_manager,
            client_pool,
            rocksdb_engine_handler,
            connection_manager,
            write_manager,
        }
    }

    pub fn awaiting_stop(
        &self,
        place_stop: Option<broadcast::Sender<bool>>,
        mqtt_stop: Option<broadcast::Sender<bool>>,
        journal_stop: Option<broadcast::Sender<bool>>,
    ) {
        self.main_runtime.block_on(async {
            self.broker_cache
                .set_status(common_base::node_status::NodeStatus::Running)
                .await;
            // Wait for all the request packets in the TCP Channel to be processed completely before starting to stop other processing threads.
            signal::ctrl_c().await.expect("failed to listen for event");
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );

            self.broker_cache
                .set_status(common_base::node_status::NodeStatus::Stopping)
                .await;

            if let Some(sx) = mqtt_stop {
                if let Err(e) = sx.send(true) {
                    error!("mqtt stop signal, error message:{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Err(e) = self.offset_manager.flush().await {
                error!(
                    "Offset manager flush operation failed. Error message: {}",
                    e
                );
            }

            if let Some(sx) = journal_stop {
                if let Err(e) = sx.send(true) {
                    error!("journal stop signal, error message{}", e);
                }
                sleep(Duration::from_secs(3));
            }

            if let Some(sx) = place_stop {
                if let Err(e) = sx.send(true) {
                    error!("place stop signal, error message{}", e);
                }
            }
            sleep(Duration::from_secs(3));
        });
    }

    fn check_grpc_server_ready(&self, grpc_port: u32, grpc_ready: Arc<AtomicBool>) {
        let max_retries = 30;
        let retry_interval = Duration::from_millis(100);

        std::thread::spawn(move || {
            let addr = format!("127.0.0.1:{grpc_port}");

            for attempt in 1..=max_retries {
                match std::net::TcpStream::connect(&addr) {
                    Ok(_) => {
                        info!("GRPC server is ready on port {grpc_port}");
                        grpc_ready.store(true, Ordering::Relaxed);
                        return;
                    }
                    Err(e) => {
                        if attempt % 10 == 0 {
                            info!(
                                "GRPC server not ready yet (attempt {}/{}): {}",
                                attempt, max_retries, e
                            );
                        }
                    }
                }

                std::thread::sleep(retry_interval);
            }
            error!(
                "GRPC server failed to start within {} attempts",
                max_retries
            );
            std::process::exit(1);
        });
    }

    fn wait_for_grpc_ready(&self, grpc_ready: &Arc<AtomicBool>) {
        let max_wait_time = Duration::from_secs(10);
        let check_interval = Duration::from_millis(100);
        let start_time = std::time::Instant::now();

        while !grpc_ready.load(Ordering::Relaxed) {
            if start_time.elapsed() > max_wait_time {
                error!("GRPC server failed to start within {:?}", max_wait_time);
                std::process::exit(1);
            }
            std::thread::sleep(check_interval);
        }

        info!("GRPC server startup check completed");
    }

    fn wait_for_journal_ready(&self) {
        let journal_port = self.config.storage_runtime.tcp_port;
        let max_wait_time = Duration::from_secs(10);
        let check_interval = Duration::from_millis(100);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < max_wait_time {
            match std::net::TcpStream::connect(format!("127.0.0.1:{journal_port}")) {
                Ok(_) => {
                    info!("Journal server startup check completed");
                    return;
                }
                Err(_) => {
                    std::thread::sleep(check_interval);
                }
            }
        }

        error!("Journal server failed to start within {:?}", max_wait_time);
        std::process::exit(1);
    }

    async fn register_node(&self, main_stop: broadcast::Sender<bool>) {
        // register node
        let client_pool = self.client_pool.clone();
        let broker_cache = self.broker_cache.clone();

        // register node
        let config = broker_config();
        match register_node(&client_pool, &broker_cache).await {
            Ok(()) => {
                // heartbeat report
                let raw_client_pool = client_pool.clone();
                tokio::spawn(async move {
                    report_heartbeat(&raw_client_pool, &broker_cache, main_stop.clone()).await;
                });

                info!("Node {} has been successfully registered", config.broker_id);
            }
            Err(e) => {
                error!("Node registration failed. Error message:{}", e);
            }
        }
    }
}
