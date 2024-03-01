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

use self::services::GrpcService;
use broker_cluster::BrokerCluster;
use clients::placement_center::PlacementCenterClientManager;
use cluster::PlacementCluster;
use common::config::placement_center::PlacementCenterConfig;
use common::log::info_meta;
use common::runtime::create_runtime;
use controller::broker_controller::BrokerServerController;
use controller::storage_controller::StorageEngineController;
use http::server::HttpServer;
use peer::{PeerMessage, PeersSender};
use protocol::placement_center::placement::placement_center_service_server::PlacementCenterServiceServer;
use raft::message::RaftMessage;
use raft::raft::MetaRaft;
use route::DataRoute;
use runtime::heartbeat::Heartbeat;
use std::fmt;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use storage::cluster_storage;
use storage::raft_core::RaftRocksDBStorageCore;
use storage::rocksdb::RocksDBStorage;
use storage_cluster::StorageCluster;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;

pub mod broker;
mod broker_cluster;
pub mod client;
pub mod cluster;
pub mod controller;
pub mod http;
mod peer;
pub mod raft;
mod route;
mod runtime;
mod services;
pub mod storage;
mod storage_cluster;
mod tools;

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
    storage_cluster: Arc<RwLock<StorageCluster>>,
    // Cache metadata information for the Broker Server cluster
    broker_cluster: Arc<RwLock<BrokerCluster>>,
    // Cache metadata information for the Placement Cluster cluster
    placement_cluster: Arc<RwLock<PlacementCluster>>,
    // Storage implementation of Raft Group information
    raft_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    // Placement Center Cluster information storage implementation
    cluster_storage: Arc<cluster_storage::ClusterStorage>,
}

impl PlacementCenter {
    pub fn new(config: PlacementCenterConfig) -> PlacementCenter {
        let server_runtime = create_runtime("server-runtime", config.runtime_work_threads);
        let daemon_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);

        let storage_cluster: Arc<RwLock<StorageCluster>> =
            Arc::new(RwLock::new(StorageCluster::new()));
        let broker_cluster: Arc<RwLock<BrokerCluster>> =
            Arc::new(RwLock::new(BrokerCluster::new()));
        let placement_cluster: Arc<RwLock<PlacementCluster>> =
            Arc::new(RwLock::new(PlacementCluster::new(
                Node::new(config.addr.clone(), config.node_id, config.grpc_port),
                config.nodes.clone(),
            )));

        let rocksdb_handle: Arc<RocksDBStorage> = Arc::new(RocksDBStorage::new(&config));
        let raft_storage: Arc<RwLock<RaftRocksDBStorageCore>> = Arc::new(RwLock::new(
            RaftRocksDBStorageCore::new(rocksdb_handle.clone()),
        ));

        let cluster_storage: Arc<cluster_storage::ClusterStorage> =
            Arc::new(cluster_storage::ClusterStorage::new(rocksdb_handle.clone()));

        return PlacementCenter {
            config,
            server_runtime,
            daemon_runtime,
            storage_cluster,
            broker_cluster,
            placement_cluster,
            raft_storage,
            cluster_storage,
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
        let http_s = HttpServer::new(
            self.config.clone(),
            self.placement_cluster.clone(),
            self.raft_storage.clone(),
            self.cluster_storage.clone(),
        );
        self.server_runtime.spawn(async move {
            http_s.start().await;
        });
    }

    // Start Grpc Server
    pub fn start_grpc_server(&self, raft_sender: Sender<RaftMessage>) {
        let ip = format!("0.0.0.0:{}", self.config.grpc_port)
            .parse()
            .unwrap();
        let service_handler = GrpcService::new(
            self.placement_cluster.clone(),
            raft_sender,
            self.raft_storage.clone(),
            self.cluster_storage.clone(),
            self.storage_cluster.clone(),
            self.broker_cluster.clone(),
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
        let storage_cluster: Arc<RwLock<StorageCluster>> = self.storage_cluster.clone();
        self.daemon_runtime.spawn(async move {
            let ctrl = StorageEngineController::new(storage_cluster);
            ctrl.start().await;
        });
        info_meta("Storage Engine Controller started successfully");
    }

    // Start Broker Server Cluster Controller
    pub fn start_broker_controller(&self) {
        let broker_cluster = self.broker_cluster.clone();
        self.daemon_runtime.spawn(async move {
            let ctrl = BrokerServerController::new(broker_cluster);
            ctrl.start().await;
        });
        info_meta("Broker Server Controller started successfully");
    }

    // Start Cluster hearbeat check thread
    pub fn start_heartbeat_check(&self, raft_sender: Sender<RaftMessage>) {
        let heartbeat = Heartbeat::new(
            100000,
            self.storage_cluster.clone(),
            self.broker_cluster.clone(),
        );
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
            self.cluster_storage.clone(),
            self.storage_cluster.clone(),
            self.broker_cluster.clone(),
        )));

        let mut raft: MetaRaft = MetaRaft::new(
            self.config.clone(),
            self.placement_cluster.clone(),
            data_route,
            peer_message_send,
            raft_message_recv,
            stop_recv,
            self.raft_storage.clone(),
        );
        self.daemon_runtime.block_on(async move {
            raft.run().await;
        });
    }

    // Start Raft Node Peer Manager
    pub fn start_peers_manager(&self, peer_message_recv: Receiver<PeerMessage>) {
        let mut peers_manager = PeersSender::new(peer_message_recv);
        self.daemon_runtime.spawn(async move {
            peers_manager.start_reveive().await;
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
