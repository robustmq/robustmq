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
use grpc_clients::pool::ClientPool;
use openraft::raft::ClientWriteResponse;
use openraft::{Config, Raft};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

pub const RAFT_STATE_MACHINE_NAME_METADATA: &str = "metadata";
pub const RAFT_STATE_MACHINE_NAME_OFFSET: &str = "offset";
pub const RAFT_STATE_MACHINE_NAME_MQTT: &str = "mqtt";

pub struct MultiRaftManager {
    pub metadata_raft_node: Raft<TypeConfig>,
    pub offset_raft_node: Raft<TypeConfig>,
    pub mqtt_raft_node: Raft<TypeConfig>,
}

impl MultiRaftManager {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        rocksdb_engine_handler: Arc<rocksdb_engine::rocksdb::RocksDBEngine>,
        route: Arc<DataRoute>,
    ) -> Result<Self, CommonError> {
        info!("Initializing Multi-Raft Manager...");

        info!("Creating Raft node: {}", RAFT_STATE_MACHINE_NAME_METADATA);
        let metadata_raft_node = MultiRaftManager::create_raft_node(
            RAFT_STATE_MACHINE_NAME_METADATA,
            &client_pool,
            &rocksdb_engine_handler,
            &route,
        )
        .await?;

        info!("Creating Raft node: {}", RAFT_STATE_MACHINE_NAME_OFFSET);
        let offset_raft_node = MultiRaftManager::create_raft_node(
            RAFT_STATE_MACHINE_NAME_OFFSET,
            &client_pool,
            &rocksdb_engine_handler,
            &route,
        )
        .await?;

        info!("Creating Raft node: {}", RAFT_STATE_MACHINE_NAME_MQTT);
        let mqtt_raft_node = MultiRaftManager::create_raft_node(
            RAFT_STATE_MACHINE_NAME_MQTT,
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
        })
    }

    pub async fn start(&self) -> Result<(), CommonError> {
        info!("Starting Multi-Raft cluster...");

        MultiRaftManager::start_raft_node(
            RAFT_STATE_MACHINE_NAME_METADATA,
            &self.metadata_raft_node,
        )
        .await?;

        MultiRaftManager::start_raft_node(RAFT_STATE_MACHINE_NAME_OFFSET, &self.offset_raft_node)
            .await?;

        MultiRaftManager::start_raft_node(RAFT_STATE_MACHINE_NAME_MQTT, &self.mqtt_raft_node)
            .await?;

        info!("Multi-Raft cluster started successfully");
        Ok(())
    }

    pub async fn write_metadata(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        Ok(Some(
            timeout(
                Duration::from_secs(5),
                self.metadata_raft_node.client_write(data),
            )
            .await??,
        ))
    }

    pub async fn write_offset(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        Ok(Some(
            timeout(
                Duration::from_secs(5),
                self.offset_raft_node.client_write(data),
            )
            .await??,
        ))
    }

    pub async fn write_mqtt(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        Ok(Some(
            timeout(
                Duration::from_secs(5),
                self.mqtt_raft_node.client_write(data),
            )
            .await??,
        ))
    }

    pub async fn shutdown(&self) -> Result<(), CommonError> {
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
        machine: &str,
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
        machine: &str,
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
        let (log_store, state_machine_store) =
            new_storage(machine, rocksdb_engine_handler.clone(), route.clone()).await;

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
