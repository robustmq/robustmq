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
    state::{HttpState, MQTTContext, NatsContext, StorageEngineContext},
};
use common_base::role::is_engine_node;
#[cfg(not(windows))]
use pprof::ProfilerGuard;
use std::sync::Arc;
use tracing::error;

use crate::{grpc::start_grpc_server, BrokerServer};

impl BrokerServer {
    pub fn start_grpc_server(&self) {
        let place_params = self.meta_params.clone();
        let mqtt_params = self.mqtt_params.clone();
        let nats_params = self.nats_params.clone();
        let engine_params = self.engine_params.clone();
        let kafka_cache = self.kafka_params.kafka_cache.clone();
        let amqp_cache = self.amqp_params.amqp_cache.clone();
        let grpc_port = self.config.grpc_port;
        self.server_runtime.spawn(Box::pin(async move {
            if let Err(e) = start_grpc_server(
                place_params,
                mqtt_params,
                nats_params,
                engine_params,
                kafka_cache,
                amqp_cache,
                grpc_port,
            )
            .await
            {
                error!("Failed to start GRPC server on port {}: {}", grpc_port, e);
                std::process::exit(1);
            }
        }));
    }

    pub fn start_admin_server(&self) {
        let broker_cache = self.broker_cache.clone();
        let nats_cache_manager = self.nats_params.cache_manager.clone();
        let nats_subscribe_manager = self.nats_params.subscribe_manager.clone();
        let nats_tcp_port = self.config.nats_runtime.tcp_port;

        #[cfg(not(windows))]
        let pprof_guard = if self.config.runtime.pprof_enable {
            match ProfilerGuard::new(100) {
                Ok(guard) => Some(Arc::new(guard)),
                Err(e) => {
                    error!("Failed to start pprof profiler: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            None
        };

        let client_pool = self.client_pool.clone();
        let connection_manager = self.mqtt_params.connection_manager.clone();
        let mqtt_context = MQTTContext {
            cache_manager: self.mqtt_params.cache_manager.clone(),
            security_manager: self.mqtt_params.security_manager.clone(),
            subscribe_manager: self.mqtt_params.subscribe_manager.clone(),
            metrics_manager: self.mqtt_params.metrics_cache_manager.clone(),
            connector_manager: self.mqtt_params.connector_manager.clone(),
            schema_manager: self.mqtt_params.schema_manager.clone(),
            push_manager: self.mqtt_params.push_manager.clone(),
            storage_driver_manager: self.mqtt_params.storage_driver_manager.clone(),
        };
        let engine_context = StorageEngineContext {
            engine_adapter_handler: self.engine_params.storage_engine_handler.clone(),
            cache_manager: self.engine_params.cache_manager.clone(),
            fetcher_manager: self.engine_params.fetcher_manager.clone(),
            memory_storage_engine: self.engine_params.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.engine_params.rocksdb_storage_engine.clone(),
        };
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let storage_driver_manager = self.mqtt_params.storage_driver_manager.clone();
        let rate_limiter = self.global_rate_limiter.clone();

        let state = Arc::new(HttpState {
            client_pool,
            connection_manager,
            mqtt_context,
            engine_context,
            rocksdb_engine_handler,
            broker_cache,
            storage_driver_manager,
            rate_limiter,
            nats_context: Some(NatsContext {
                cache_manager: nats_cache_manager,
                subscribe_manager: nats_subscribe_manager,
                nats_tcp_port,
            }),
            #[cfg(not(windows))]
            pprof_guard,
        });

        let http_port = self.config.http_port;
        self.server_runtime.spawn(async move {
            if let Err(e) = AdminServer::new().start(http_port, state).await {
                error!(
                    "Admin HTTP server failed to start on port {}: {}",
                    http_port, e
                );
                std::process::exit(1);
            }
        });
    }

    pub fn start_load_cache(&self) {
        let mqtt_cache_manager = self.mqtt_params.cache_manager.clone();
        let nats_subscribe_manager = self.nats_params.subscribe_manager.clone();
        let nats_cache_manager = self.nats_params.cache_manager.clone();
        let client_pool = self.client_pool.clone();
        let connector_manager = self.mqtt_params.connector_manager.clone();
        let schema_manager = self.mqtt_params.schema_manager.clone();
        let security_manager = self.mqtt_params.security_manager.clone();
        let kafka_cache = self.kafka_params.kafka_cache.clone();
        let amqp_cache = self.amqp_params.amqp_cache.clone();
        self.server_runtime.block_on(async {
            if let Err(e) = crate::load_cache::load_metadata_cache(
                &mqtt_cache_manager,
                &nats_subscribe_manager,
                &nats_cache_manager,
                &client_pool,
                &connector_manager,
                &schema_manager,
                &security_manager,
                &kafka_cache,
                &amqp_cache,
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
