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

use self::server::peer::{PeerMessage, PeersManager};
use cache::broker_cluster::BrokerClusterCache;
use cache::engine_cluster::EngineClusterCache;
use cache::placement_cluster::PlacementClusterCache;
use clients::ClientPool;
use common::config::placement_center::PlacementCenterConfig;
use common::log::info_meta;
use common::runtime::create_runtime;
use controller::broker_controller::BrokerServerController;
use controller::storage_controller::StorageEngineController;
use http::server::{start_http_server, HttpServerState};
use protocol::placement_center::placement::placement_center_service_server::PlacementCenterServiceServer;
use raft::message::RaftMessage;
use raft::raft::RaftMachine;
use raft::route::DataRoute;
use rocksdb::raft::RaftMachineStorage;
use rocksdb::rocksdb::RocksDBEngine;
use server::grpc::GrpcService;
use server::heartbeat::Heartbeat;
use std::fmt;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Mutex};
use tonic::transport::Server;

mod broker;
mod cache;
mod controller;
mod http;
mod raft;
mod rocksdb;
mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Node {
    pub ip: String,
    pub id: u64,
    pub inner_port: u16,
}

impl Node {
    pub fn new(ip: String, id: u64, port: u16) -> Node {
        Node {
            ip,
            id,
            inner_port: port,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.inner_port)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ip:{},id:{},port:{}", self.ip, self.id, self.inner_port)
    }
}

pub struct PlacementCenter {
    config: PlacementCenterConfig,

    server_runtime: Runtime,
    daemon_runtime: Runtime,
    // Cache metadata information for the Storage Engine cluster
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    // Cache metadata information for the Broker Server cluster
    broker_cache: Arc<RwLock<BrokerClusterCache>>,
    // Cache metadata information for the Placement Cluster cluster
    placement_cache: Arc<RwLock<PlacementClusterCache>>,
    // Storage implementation of Raft Group information
    raft_machine_storage: Arc<RwLock<RaftMachineStorage>>,
    //
    rocksdb_engine: Arc<RocksDBEngine>,
    client_poll: Arc<Mutex<ClientPool>>,
}

impl PlacementCenter {
    pub fn new(config: PlacementCenterConfig) -> PlacementCenter {
        let server_runtime = create_runtime("server-runtime", config.runtime_work_threads);
        let daemon_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);

        let engine_cache = Arc::new(RwLock::new(EngineClusterCache::new()));
        let broker_cache = Arc::new(RwLock::new(BrokerClusterCache::new()));
        let placement_cache = Arc::new(RwLock::new(PlacementClusterCache::new(
            Node::new(config.addr.clone(), config.node_id, config.grpc_port),
            config.nodes.clone(),
        )));

        let rocksdb_engine: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&config));
        let raft_machine_storage =
            Arc::new(RwLock::new(RaftMachineStorage::new(rocksdb_engine.clone())));

        let client_poll = Arc::new(Mutex::new(ClientPool::new()));

        return PlacementCenter {
            config,
            server_runtime,
            daemon_runtime,
            engine_cache,
            broker_cache,
            placement_cache,
            raft_machine_storage,
            rocksdb_engine,
            client_poll,
        };
    }

    pub fn start(&mut self, stop_send: broadcast::Sender<bool>, is_banner: bool) {
        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
        let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);

        let stop_recv = stop_send.subscribe();

        self.start_broker_controller();

        self.start_engine_controller();

        self.start_peers_manager(peer_message_recv);

        self.start_http_server();

        self.start_heartbeat_check(raft_message_send.clone());

        self.start_grpc_server(raft_message_send);

        self.awaiting_stop(stop_send);

        self.start_raft_machine(peer_message_send, raft_message_recv, stop_recv, is_banner);
    }

    // Start HTTP Server
    pub fn start_http_server(&self) {
        let state: HttpServerState = HttpServerState::new(
            self.placement_cache.clone(),
            self.raft_machine_storage.clone(),
            self.engine_cache.clone(),
        );
        let port = self.config.http_port;
        self.server_runtime.spawn(async move {
            start_http_server(port, state).await;
        });
    }

    // Start Grpc Server
    pub fn start_grpc_server(&self, raft_sender: Sender<RaftMessage>) {
        let ip = format!("0.0.0.0:{}", self.config.grpc_port)
            .parse()
            .unwrap();
        let service_handler = GrpcService::new(
            self.placement_cache.clone(),
            raft_sender,
            self.raft_machine_storage.clone(),
            self.engine_cache.clone(),
            self.broker_cache.clone(),
            self.client_poll.clone(),
        );
        self.server_runtime.spawn(async move {
            info_meta(&format!(
                "RobustMQ Meta Grpc Server start success. bind addr:{}",
                ip
            ));
            Server::builder()
                .add_service(PlacementCenterServiceServer::new(service_handler))
                .serve(ip)
                .await
                .unwrap();
        });
    }

    // Start Storage Engine Cluster Controller
    pub fn start_engine_controller(&self) {
        let engine_cache = self.engine_cache.clone();
        self.daemon_runtime.spawn(async move {
            let ctrl = StorageEngineController::new(engine_cache);
            ctrl.start().await;
        });
        info_meta("Storage Engine Controller started successfully");
    }

    // Start Broker Server Cluster Controller
    pub fn start_broker_controller(&self) {
        let broker_cache = self.broker_cache.clone();
        self.daemon_runtime.spawn(async move {
            let ctrl = BrokerServerController::new(broker_cache);
            ctrl.start().await;
        });
        info_meta("Broker Server Controller started successfully");
    }

    // Start Cluster hearbeat check thread
    pub fn start_heartbeat_check(&self, raft_sender: Sender<RaftMessage>) {
        let heartbeat =
            Heartbeat::new(100000, self.engine_cache.clone(), self.broker_cache.clone());
        self.daemon_runtime.spawn(async move {
            heartbeat.start_heartbeat_check().await;
        });

        info_meta("Cluster node heartbeat detection thread started successfully");
    }

    // Start Raft Status Machine
    pub fn start_raft_machine(
        &self,
        peer_message_send: Sender<PeerMessage>,
        raft_message_recv: Receiver<RaftMessage>,
        stop_recv: broadcast::Receiver<bool>,
        is_banner: bool,
    ) {
        let data_route = Arc::new(RwLock::new(DataRoute::new(
            self.rocksdb_engine.clone(),
            self.engine_cache.clone(),
            self.broker_cache.clone(),
        )));

        let mut raft: RaftMachine = RaftMachine::new(
            self.config.clone(),
            self.placement_cache.clone(),
            data_route,
            peer_message_send,
            raft_message_recv,
            stop_recv,
            self.raft_machine_storage.clone(),
        );
        self.daemon_runtime.block_on(async move {
            raft.run().await;
        });
    }

    // Start Raft Node Peer Manager
    pub fn start_peers_manager(&self, peer_message_recv: Receiver<PeerMessage>) {
        let mut peers_manager = PeersManager::new(peer_message_recv, self.client_poll.clone());
        self.daemon_runtime.spawn(async move {
            peers_manager.start().await;
        });

        info_meta("Raft Node inter-node communication management thread started successfully");
    }

    // Wait Stop Signal
    pub fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        self.daemon_runtime.spawn(async move {
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
    }
}
