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

use super::network::network::Network;
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
use rocksdb_engine::storage::family::storage_raft_fold;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

pub struct MultiRaftManager {
    pub metadata_raft_node: Raft<TypeConfig>,
    offset_raft_node: Raft<TypeConfig>,
    mqtt_raft_node: Raft<TypeConfig>,
}

impl MultiRaftManager {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        route: Arc<DataRoute>,
    ) -> Result<Self, CommonError> {
        let metadata_raft_node =
            MultiRaftManager::create_raft_node("metadata", &client_pool, &route).await?;

        let offset_raft_node =
            MultiRaftManager::create_raft_node("offset", &client_pool, &route).await?;

        let mqtt_raft_node =
            MultiRaftManager::create_raft_node("mqtt", &client_pool, &route).await?;
        Ok(MultiRaftManager {
            metadata_raft_node,
            offset_raft_node,
            mqtt_raft_node,
        })
    }

    pub async fn start(&self) -> Result<(), CommonError> {
        MultiRaftManager::start_raft_node("metadata", &self.metadata_raft_node).await?;
        MultiRaftManager::start_raft_node("offset", &self.offset_raft_node).await?;
        MultiRaftManager::start_raft_node("mqtt", &self.mqtt_raft_node).await?;
        Ok(())
    }

    pub async fn write_metadata(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        Ok(Some(
            timeout(
                Duration::from_secs(10),
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
                Duration::from_secs(10),
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
                Duration::from_secs(10),
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
        let conf = broker_config();
        let mut nodes = BTreeMap::new();
        for (node_id, addr) in conf.meta_addrs.clone() {
            let mut addr = addr.to_string();
            addr = addr.replace("\"", "");
            let node = Node {
                rpc_addr: addr,
                node_id: node_id.parse().unwrap(),
            };

            nodes.insert(node.node_id, node);
        }

        info!("[{}],Raft Nodes:{:?}", machine, nodes);

        match raft_node.is_initialized().await {
            Ok(flag) => {
                info!(
                    "[{}],Whether nodes should be initialized, flag={}",
                    machine, flag
                );
                if !flag {
                    match raft_node.initialize(nodes.clone()).await {
                        Ok(_) => {
                            info!(
                                "[{}],Node {:?} was initialized successfully",
                                machine, nodes
                            );
                        }
                        Err(e) => {
                            return Err(CommonError::CommonError(format!(
                                "[{machine}], openraft init fail,{e}"
                            )));
                        }
                    }
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "[{machine}], openraft initialized fail,{e}"
                )));
            }
        }
        Ok(())
    }

    async fn create_raft_node(
        machine: &str,
        client_pool: &Arc<ClientPool>,
        route: &Arc<DataRoute>,
    ) -> Result<Raft<TypeConfig>, CommonError> {
        let config = Config {
            heartbeat_interval: 250,
            election_timeout_min: 299,
            allow_log_reversion: Some(true),
            ..Default::default()
        };

        let config = Arc::new(match config.validate() {
            Ok(data) => data,
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        });

        let conf = broker_config();
        let path = storage_raft_fold(&conf.rocksdb.data_path);
        let dir = Path::new(&path);
        let (log_store, state_machine_store) = new_storage(&dir, route.clone()).await;

        let network = Network::new(machine.to_string(), client_pool.clone());

        match Raft::new(
            conf.broker_id,
            config.clone(),
            network,
            log_store,
            state_machine_store,
        )
        .await
        {
            Ok(data) => return Ok(data),
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "[{machine}],Failed to initialize raft node with error message :{e}"
                )));
            }
        }
    }
}
