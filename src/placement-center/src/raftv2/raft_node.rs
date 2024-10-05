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

use crate::storage::route::DataRoute;

use super::network::network::Network;
use super::store::new_storage;
use super::typeconfig::TypeConfig;
use clients::poll::ClientPool;
use common_base::config::placement_center::placement_center_conf;
use log::info;
use openraft::{Config, Raft};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;
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

pub mod typ {
    use openraft::error::Infallible;

    use crate::raftv2::typeconfig::TypeConfig;

    pub type Entry = openraft::Entry<TypeConfig>;

    pub type RaftError<E = Infallible> = openraft::error::RaftError<TypeConfig, E>;
    pub type RPCError<E = Infallible> = openraft::error::RPCError<TypeConfig, RaftError<E>>;

    pub type ClientWriteError = openraft::error::ClientWriteError<TypeConfig>;
    pub type CheckIsLeaderError = openraft::error::CheckIsLeaderError<TypeConfig>;
    pub type ForwardToLeader = openraft::error::ForwardToLeader<TypeConfig>;
    pub type InitializeError = openraft::error::InitializeError<TypeConfig>;

    pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;
}

pub type ExampleRaft = openraft::Raft<TypeConfig>;

pub async fn start_openraft_node(raft_node: Raft<TypeConfig>) {
    let conf = placement_center_conf();
    let mut nodes = BTreeMap::new();
    for (node_id, addr) in conf.node.nodes.clone() {
        let mut addr = addr.to_string();
        addr = addr.replace("\"", "");
        let node = Node {
            rpc_addr: addr,
            node_id: node_id.parse().unwrap(),
        };

        nodes.insert(node.node_id, node);
    }

    info!("Raft Nodes:{:?}", nodes);
    let init_node_id = calc_init_node(&nodes);
    if init_node_id == conf.node.node_id {
        match raft_node.is_initialized().await {
            Ok(flag) => {
                info!("Whether nodes should be initialized, flag={}", flag);
                if !flag {
                    match raft_node.initialize(nodes.clone()).await {
                        Ok(_) => {
                            info!("Node {:?} was initialized successfully", nodes);
                        }
                        Err(e) => {
                            panic!("openraft init fail,{}", e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                panic!("openraft initialized fail,{}", e.to_string());
            }
        }
    }
}

pub fn calc_init_node(nodes: &BTreeMap<u64, Node>) -> u64 {
    let mut node_ids: Vec<u64> = nodes.keys().map(|x| x.clone()).collect();
    node_ids.sort();
    return node_ids.first().unwrap().clone();
}

pub async fn create_raft_node(
    client_poll: Arc<ClientPool>,
    route: Arc<DataRoute>,
) -> Raft<TypeConfig> {
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());
    let conf = placement_center_conf();
    let path = format!("{}/_engine_storage", conf.rocksdb.data_path.clone());
    let dir = Path::new(&path);
    let (log_store, state_machine_store) = new_storage(&dir, route).await;

    let network = Network::new(client_poll);
    let raft = openraft::Raft::new(
        conf.node.node_id,
        config.clone(),
        network,
        log_store,
        state_machine_store,
    )
    .await
    .unwrap();

    return raft;
}
