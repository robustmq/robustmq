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

use std::sync::{Arc, RwLock};

use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info_meta,
    metrics::register_prometheus_export,
    runtime::create_runtime,
};
use metadata::cache::MetadataCache;
use server::{grpc::server::GrpcServer, start_mqtt_server};
use tokio::{runtime::Runtime, signal, sync::broadcast};

mod metadata;
mod packet;
mod server;
mod storage;

pub struct MqttBroker<'a> {
    conf: &'a BrokerMQTTConfig,
    metadata_cache: Arc<RwLock<MetadataCache>>,
    runtime: Runtime,
}

impl<'a> MqttBroker<'a> {
    pub fn new() -> Self {
        let conf = broker_mqtt_conf();
        let runtime = create_runtime("storage-engine-server-runtime", conf.runtime.worker_threads);
        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new()));
        return MqttBroker {
            conf,
            runtime,
            metadata_cache,
        };
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        self.start_grpc_server();
        self.start_mqtt_server();
        self.start_prometheus_export();
        self.awaiting_stop(stop_send);
    }

    fn start_mqtt_server(&self) {
        self.runtime.spawn(async move { start_mqtt_server().await });
    }

    fn start_grpc_server(&self) {
        let port = self.conf.port.clone();
        let cache = self.metadata_cache.clone();
        self.runtime.spawn(async move {
            let server = GrpcServer::new(port, cache);
            server.start().await;
        });
    }

    fn start_prometheus_export(&self) {
        if self.conf.prometheus.enable {
            let port = self.conf.prometheus.port.clone();
            self.runtime
                .spawn(async move { register_prometheus_export(port).await });
        }
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
