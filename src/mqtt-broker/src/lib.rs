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
    log::info_meta,
    runtime::create_runtime,
};
use flume::{Receiver, Sender};
use metadata::{cache::MetadataCache, hearbeat::HeartbeatManager};
use server::{
    grpc::server::GrpcServer,
    http::server::{start_http_server, HttpServerState},
    start_mqtt_server,
    tcp::packet::ResponsePackage,
};
use std::sync::{Arc, RwLock};
use subscribe::subscribe_manager::SubScribeManager;
use tokio::{runtime::Runtime, signal, sync::broadcast};

mod metadata;
mod metrics;
mod packet;
mod security;
mod server;
mod storage;
mod subscribe;

pub struct MqttBroker<'a> {
    conf: &'a BrokerMQTTConfig,
    metadata_cache: Arc<RwLock<MetadataCache>>,
    heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    runtime: Runtime,
}

impl<'a> MqttBroker<'a> {
    pub fn new() -> Self {
        let conf = broker_mqtt_conf();
        let runtime = create_runtime("storage-engine-server-runtime", conf.runtime.worker_threads);
        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new()));
        let heartbeat_manager = Arc::new(RwLock::new(HeartbeatManager::new()));

        let subscribe_manager = Arc::new(RwLock::new(SubScribeManager::new()));

        return MqttBroker {
            conf,
            runtime,
            metadata_cache,
            heartbeat_manager,
            subscribe_manager,
        };
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.start_grpc_server();
        self.start_mqtt_server();
        self.start_prometheus_export();
        self.awaiting_stop(stop_send);
    }

    fn start_mqtt_server(&self) {
        let cache = self.metadata_cache.clone();
        let heartbeat_manager = self.heartbeat_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        self.runtime.spawn(async move {
            start_mqtt_server(cache, heartbeat_manager, subscribe_manager).await
        });
    }

    fn start_grpc_server(&self) {
        let port = self.conf.grpc_port.clone();
        let cache = self.metadata_cache.clone();
        self.runtime.spawn(async move {
            let server = GrpcServer::new(port, cache);
            server.start().await;
        });
    }

    fn start_prometheus_export(&self) {
        let http_state = HttpServerState::new(
            self.metadata_cache.clone(),
            self.heartbeat_manager.clone(),
            self.subscribe_manager.clone(),
        );
        self.runtime
            .spawn(async move { start_http_server(http_state).await });
    }

    fn start_websocket_server(&self) {}

    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        // Wait for the stop signal
        self.runtime.block_on(async move {
            loop {
                signal::ctrl_c().await.expect("failed to listen for event");
                match stop_send.send(true) {
                    Ok(_) => {
                        info_meta("When ctrl + c is received, the service starts to stop");
                        break;
                    }
                    Err(_) => {}
                }
            }
        });

        // todo tokio runtime shutdown
    }
}
