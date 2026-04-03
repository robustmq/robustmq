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

use admin_server::{
    server::AdminServer,
    state::{HttpState, MQTTContext, StorageEngineContext},
};
use common_base::role::is_engine_node;
use common_metrics::core::server::register_prometheus_export;
use pprof_monitor::pprof_monitor::start_pprof_monitor;
use std::sync::Arc;
use tracing::error;

use crate::{grpc::start_grpc_server, BrokerServer};

impl BrokerServer {
    pub fn start_grpc_server(&self) {
        let place_params = self.meta_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let engine_params = self.engine_params.clone();
        let grpc_port = self.config.grpc_port;
        self.server_runtime.spawn(Box::pin(async move {
            if let Err(e) =
                start_grpc_server(place_params, mqtt_params, engine_params, grpc_port).await
            {
                error!("Failed to start GRPC server: {}", e);
                std::process::exit(1);
            }
        }));
    }

    pub fn start_admin_server(&self) {
        let broker_cache = self.broker_cache.clone();
        let state = Arc::new(HttpState {
            client_pool: self.client_pool.clone(),
            connection_manager: self.mqtt_params.connection_manager.clone(),
            mqtt_context: MQTTContext {
                cache_manager: self.mqtt_params.cache_manager.clone(),
                security_manager: self.mqtt_params.security_manager.clone(),
                subscribe_manager: self.mqtt_params.subscribe_manager.clone(),
                metrics_manager: self.mqtt_params.metrics_cache_manager.clone(),
                connector_manager: self.mqtt_params.connector_manager.clone(),
                schema_manager: self.mqtt_params.schema_manager.clone(),
                retain_message_manager: self.mqtt_params.retain_message_manager.clone(),
                push_manager: self.mqtt_params.push_manager.clone(),
                storage_driver_manager: self.mqtt_params.storage_driver_manager.clone(),
            },
            engine_context: StorageEngineContext {
                engine_adapter_handler: self.engine_params.storage_engine_handler.clone(),
                cache_manager: self.engine_params.cache_manager.clone(),
            },
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            broker_cache,
            storage_driver_manager: self.mqtt_params.storage_driver_manager.clone(),
            rate_limiter: self.global_rate_limiter.clone(),
        });
        let http_port = self.config.http_port;
        self.server_runtime.spawn(async move {
            AdminServer::new().start(http_port, state).await;
        });
    }

    pub fn start_pprof_server(&self) {
        let pprof_port = self.config.pprof.port;
        let pprof_frequency = self.config.pprof.frequency;
        if self.config.pprof.enable {
            self.server_runtime.spawn(async move {
                start_pprof_monitor(pprof_port, pprof_frequency).await;
            });
        }
    }

    pub fn start_prometheus_server(&self) {
        let prometheus_port = self.config.prometheus.port;
        if self.config.prometheus.enable {
            self.server_runtime.spawn(async move {
                register_prometheus_export(prometheus_port).await;
            });
        }
    }

    pub fn start_load_cache(&self) {
        let mqtt_cache_manager = self.mqtt_params.cache_manager.clone();
        let nats_cache_manager = self.nats_params.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let connector_manager = self.mqtt_params.connector_manager.clone();
        let schema_manager = self.mqtt_params.schema_manager.clone();
        self.server_runtime.block_on(async {
            if let Err(e) = crate::load_cache::load_metadata_cache(
                &mqtt_cache_manager,
                &nats_cache_manager,
                &client_pool,
                &connector_manager,
                &schema_manager,
            )
            .await
            {
                error!("Failed to load metadata cache: {}", e);
                std::process::exit(1);
            }
        });

        if is_engine_node(&self.config.roles) {
            let engine_cache = self.engine_params.cache_manager.clone();
            let client_pool = self.client_pool.clone();
            self.server_runtime.block_on(async {
                if let Err(e) =
                    crate::load_cache::load_engine_cache(&engine_cache, &client_pool).await
                {
                    error!("Failed to load engine cache: {}", e);
                    std::process::exit(1);
                }
            });
        }
    }
}
