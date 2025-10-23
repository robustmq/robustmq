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
use crate::raft::route::DataRoute;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use openraft::{Config, Raft};
use rocksdb_engine::storage::family::storage_raft_fold;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub node_id: u64,
    pub rpc_addr: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ rpc_addr: {}, node_id: {} }}",
            self.rpc_addr, self.node_id
        )
    }
}

pub mod types {
    use crate::raft::type_config::TypeConfig;
    pub type Entry = openraft::Entry<TypeConfig>;
}

pub async fn start_raft_node(raft_node: Raft<TypeConfig>) {
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

    info!("Raft Nodes:{:?}", nodes);

    match raft_node.is_initialized().await {
        Ok(flag) => {
            info!("Whether nodes should be initialized, flag={}", flag);
            if !flag {
                match raft_node.initialize(nodes.clone()).await {
                    Ok(_) => {
                        info!("Node {:?} was initialized successfully", nodes);
                    }
                    Err(e) => {
                        panic!("openraft init fail,{e}");
                    }
                }
            }
        }
        Err(e) => {
            panic!("openraft initialized fail,{e}");
        }
    }
}

pub async fn create_raft_node(
    client_pool: Arc<ClientPool>,
    route: Arc<DataRoute>,
) -> Raft<TypeConfig> {
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        allow_log_reversion: Some(true),
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());
    let conf = broker_config();
    let path = storage_raft_fold(&conf.rocksdb.data_path);
    let dir = Path::new(&path);
    let (log_store, state_machine_store) = new_storage(&dir, route).await;

    let network = Network::new(client_pool);

    match Raft::new(
        conf.broker_id,
        config.clone(),
        network,
        log_store,
        state_machine_store,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            panic!("Failed to initialize raft node with error message :{e}");
        }
    }
}
