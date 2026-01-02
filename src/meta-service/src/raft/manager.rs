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

use super::network::client::Network;
use super::store::new_storage;
use super::type_config::TypeConfig;
use crate::core::error::MetaServiceError;
use crate::raft::route::data::StorageData;
use crate::raft::route::DataRoute;
use crate::raft::type_config::Node;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_metrics::meta::raft::{
    record_write_duration, record_write_failure, record_write_request, record_write_success,
};
use grpc_clients::pool::ClientPool;
use openraft::raft::ClientWriteResponse;
use openraft::{Config, Raft};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::info;

const DEFAULT_RAFT_WRITE_TIMEOUT_SEC: u64 = 30;

#[derive(Clone, Debug)]
pub enum RaftStateMachineName {
    METADATA,
    OFFSET,
    MQTT,
}

impl RaftStateMachineName {
    pub fn as_str(&self) -> &str {
        match self {
            RaftStateMachineName::METADATA => "metadata",
            RaftStateMachineName::OFFSET => "offset",
            RaftStateMachineName::MQTT => "mqtt",
        }
    }
}

impl std::str::FromStr for RaftStateMachineName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata" => Ok(RaftStateMachineName::METADATA),
            "offset" => Ok(RaftStateMachineName::OFFSET),
            "mqtt" => Ok(RaftStateMachineName::MQTT),
            _ => Err(format!("Invalid RaftStateMachineName: {}", s)),
        }
    }
}

impl std::fmt::Display for RaftStateMachineName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct MultiRaftManager {
    pub metadata_raft_node: Raft<TypeConfig>,
    pub offset_raft_node: Raft<TypeConfig>,
    pub mqtt_raft_node: Raft<TypeConfig>,
    pub stop: Arc<RwLock<bool>>,
}

impl MultiRaftManager {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: Arc<DataRoute>,
    ) -> Result<Self, CommonError> {
        info!("Initializing Multi-Raft Manager...");

        info!("Creating Raft node: {}", RaftStateMachineName::METADATA);
        let metadata_raft_node = MultiRaftManager::create_raft_node(
            &RaftStateMachineName::METADATA,
            &client_pool,
            &rocksdb_engine_handler,
            &route,
        )
        .await?;

        info!("Creating Raft node: {}", RaftStateMachineName::OFFSET);
        let offset_raft_node = MultiRaftManager::create_raft_node(
            &RaftStateMachineName::OFFSET,
            &client_pool,
            &rocksdb_engine_handler,
            &route,
        )
        .await?;

        info!("Creating Raft node: {}", RaftStateMachineName::MQTT);
        let mqtt_raft_node = MultiRaftManager::create_raft_node(
            &RaftStateMachineName::MQTT,
            &client_pool,
            &rocksdb_engine_handler,
            &route,
        )
        .await?;

        info!("Multi-Raft Manager initialized successfully");
        Ok(MultiRaftManager {
            metadata_raft_node,
            offset_raft_node,
            mqtt_raft_node,
            stop: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<(), CommonError> {
        info!("Starting Multi-Raft cluster...");

        MultiRaftManager::start_raft_node(
            &RaftStateMachineName::METADATA,
            &self.metadata_raft_node,
        )
        .await?;

        MultiRaftManager::start_raft_node(&RaftStateMachineName::OFFSET, &self.offset_raft_node)
            .await?;

        MultiRaftManager::start_raft_node(&RaftStateMachineName::MQTT, &self.mqtt_raft_node)
            .await?;

        info!("Multi-Raft cluster started successfully");
        Ok(())
    }

    fn get_raft_write_timeout(&self) -> Duration {
        let conf = broker_config();
        Duration::from_secs(
            conf.meta_runtime
                .raft_write_timeout_sec
                .max(DEFAULT_RAFT_WRITE_TIMEOUT_SEC),
        )
    }

    pub async fn write_metadata(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        let stop = self.stop.read().await;
        if *stop {
            return Err(MetaServiceError::RaftNodeHasStopped(
                RaftStateMachineName::METADATA.to_string(),
            ));
        }

        let machine = RaftStateMachineName::METADATA.as_str();
        record_write_request(machine);
        let start = Instant::now();

        let result = timeout(
            self.get_raft_write_timeout(),
            self.metadata_raft_node.client_write(data),
        )
        .await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_write_duration(machine, duration_ms);

        match result {
            Ok(Ok(response)) => {
                record_write_success(machine);
                Ok(Some(response))
            }
            Ok(Err(e)) => {
                record_write_failure(machine);
                Err(e.into())
            }
            Err(_) => {
                record_write_failure(machine);
                Err(MetaServiceError::CommonError(
                    "Write metadata timeout".to_string(),
                ))
            }
        }
    }

    pub async fn write_offset(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        let stop = self.stop.read().await;
        if *stop {
            return Err(MetaServiceError::RaftNodeHasStopped(
                RaftStateMachineName::OFFSET.to_string(),
            ));
        }

        let machine = RaftStateMachineName::OFFSET.as_str();
        record_write_request(machine);
        let start = Instant::now();

        let result = timeout(
            self.get_raft_write_timeout(),
            self.offset_raft_node.client_write(data),
        )
        .await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_write_duration(machine, duration_ms);

        match result {
            Ok(Ok(response)) => {
                record_write_success(machine);
                Ok(Some(response))
            }
            Ok(Err(e)) => {
                record_write_failure(machine);
                Err(e.into())
            }
            Err(_) => {
                record_write_failure(machine);
                Err(MetaServiceError::CommonError(
                    "Write offset timeout".to_string(),
                ))
            }
        }
    }

    pub async fn write_mqtt(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        let stop = self.stop.read().await;
        if *stop {
            return Err(MetaServiceError::RaftNodeHasStopped(
                RaftStateMachineName::MQTT.to_string(),
            ));
        }

        let machine = RaftStateMachineName::MQTT.as_str();
        record_write_request(machine);
        let start = Instant::now();

        let result = timeout(
            self.get_raft_write_timeout(),
            self.mqtt_raft_node.client_write(data),
        )
        .await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_write_duration(machine, duration_ms);

        match result {
            Ok(Ok(response)) => {
                record_write_success(machine);
                Ok(Some(response))
            }
            Ok(Err(e)) => {
                record_write_failure(machine);
                Err(e.into())
            }
            Err(_) => {
                record_write_failure(machine);
                Err(MetaServiceError::CommonError(
                    "Write mqtt timeout".to_string(),
                ))
            }
        }
    }

    pub async fn shutdown(&self) -> Result<(), CommonError> {
        let mut write = self.stop.write().await;
        *write = true;

        self.mqtt_raft_node
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        self.offset_raft_node
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        self.metadata_raft_node
            .shutdown()
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;
        Ok(())
    }

    async fn start_raft_node(
        machine: &RaftStateMachineName,
        raft_node: &Raft<TypeConfig>,
    ) -> Result<(), CommonError> {
        info!("[{}] Starting Raft node...", machine);

        let conf = broker_config();
        let mut nodes = BTreeMap::new();

        // Build node list
        for (node_id, addr) in conf.meta_addrs.clone() {
            let addr = addr.to_string().replace("\"", "");
            let node = Node {
                rpc_addr: addr.clone(),
                node_id: node_id.parse().unwrap(),
            };
            nodes.insert(node.node_id, node);
        }

        // Print cluster members
        let node_list: Vec<String> = nodes
            .iter()
            .map(|(id, node)| format!("node_{}={}", id, node.rpc_addr))
            .collect();
        info!("[{}] Cluster members: [{}]", machine, node_list.join(", "));

        // Check if already initialized
        match raft_node.is_initialized().await {
            Ok(is_initialized) => {
                if !is_initialized {
                    info!(
                        "[{}] Initializing Raft cluster with {} nodes...",
                        machine,
                        nodes.len()
                    );

                    match raft_node.initialize(nodes.clone()).await {
                        Ok(_) => {
                            info!("[{}] Raft cluster initialized successfully", machine);
                        }
                        Err(e) => {
                            return Err(CommonError::CommonError(format!(
                                "[{}] Failed to initialize Raft cluster: {}",
                                machine, e
                            )));
                        }
                    }
                } else {
                    info!("[{}] Raft cluster already initialized, skipping", machine);
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "[{}] Failed to check initialization status: {}",
                    machine, e
                )));
            }
        }

        info!("[{}] Raft node started successfully", machine);
        Ok(())
    }

    async fn create_raft_node(
        machine: &RaftStateMachineName,
        client_pool: &Arc<ClientPool>,
        rocksdb_engine_handler: &Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: &Arc<DataRoute>,
    ) -> Result<Raft<TypeConfig>, CommonError> {
        // Raft configuration
        let config = Config {
            heartbeat_interval: 250,
            election_timeout_min: 299,
            allow_log_reversion: Some(true),
            ..Default::default()
        };

        let config = Arc::new(match config.validate() {
            Ok(data) => data,
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "[{}] Invalid Raft configuration: {}",
                    machine, e
                )));
            }
        });

        info!(
            "[{}] Raft config: heartbeat={}ms, election_timeout={}ms",
            machine, config.heartbeat_interval, config.election_timeout_min
        );

        let conf = broker_config();

        // Create storage layer (log store + state machine)
        info!(
            "[{}] Initializing storage (log + state machine)...",
            machine
        );
        let (log_store, state_machine_store) = new_storage(
            machine.as_str(),
            rocksdb_engine_handler.clone(),
            route.clone(),
        )
        .await;

        // Create network layer
        let network = Network::new(machine.to_string(), client_pool.clone());

        // Create Raft instance
        info!(
            "[{}] Creating Raft instance (node_id={})...",
            machine, conf.broker_id
        );
        match Raft::new(
            conf.broker_id,
            config.clone(),
            network,
            log_store,
            state_machine_store,
        )
        .await
        {
            Ok(raft_node) => {
                info!("[{}] Raft instance created successfully", machine);
                Ok(raft_node)
            }
            Err(e) => Err(CommonError::CommonError(format!(
                "[{}] Failed to create Raft instance: {}",
                machine, e
            ))),
        }
    }
}
