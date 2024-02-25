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
use cluster::Cluster;
use common::config::placement_center::PlacementCenterConfig;
use common::log::{info, info_meta};
use common::runtime::create_runtime;
use controller::broker_controller::BrokerServerController;
use controller::storage_controller::StorageEngineController;
use http::server::HttpServer;
use protocol::placement_center::placement::meta_service_server::MetaServiceServer;
use raft::message::RaftMessage;
use raft::peer::{PeerMessage, PeersManager};
use raft::raft::MetaRaft;
use route::DataRoute;
use runtime::heartbeat::Heartbeat;
use std::fmt;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use storage::cluster_storage;
use storage::raft_core::RaftRocksDBStorageCore;
use storage::rocksdb::RocksDBStorage;
use storage_cluster::StorageCluster;
use tokio::signal;
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;

pub mod broker;
mod broker_cluster;
pub mod client;
pub mod cluster;
pub mod controller;
mod errors;
pub mod http;
pub mod raft;
mod route;
mod runtime;
mod services;
pub mod storage;
mod storage_cluster;
mod tools;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
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
}

impl PlacementCenter {
    pub fn new(config: PlacementCenterConfig) -> PlacementCenter {
        return PlacementCenter { config };
    }

    pub fn run(
        &mut self,
        stop_send: broadcast::Sender<bool>,
    ) -> Vec<Result<JoinHandle<()>, std::io::Error>> {
        info(&format!(
            "Meta node starting, node IP :{}, node ID:{}",
            self.config.addr, self.config.node_id
        ));

        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
        let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);
        let rds: Arc<RocksDBStorage> = Arc::new(RocksDBStorage::new(&self.config));
        let rocksdb_storage = Arc::new(RwLock::new(RaftRocksDBStorageCore::new(rds.clone())));
        let cluster_storage = Arc::new(cluster_storage::ClusterStorage::new(rds.clone()));

        let cluster = Arc::new(RwLock::new(Cluster::new(
            Node::new(
                self.config.addr.clone(),
                self.config.node_id.clone(),
                self.config.port.clone(),
            ),
            peer_message_send.clone(),
            self.config.nodes.clone(),
        )));

        let storage_cluster = Arc::new(RwLock::new(StorageCluster::new()));
        let broker_cluster = Arc::new(RwLock::new(BrokerCluster::new()));

        let data_route = Arc::new(RwLock::new(DataRoute::new(
            cluster_storage.clone(),
            storage_cluster.clone(),
            broker_cluster.clone(),
        )));
        let mut thread_result = Vec::new();

        let heartbeat = Heartbeat::new(100000, storage_cluster.clone(), broker_cluster.clone());

        // Thread running meta tcp server
        let tcp_thread = thread::Builder::new().name("meta-tcp-thread".to_owned());
        let config = self.config.clone();
        let cluster_clone = cluster.clone();
        let mut stop_recv_c = stop_send.subscribe();
        let rocksdb_storage_c = rocksdb_storage.clone();
        let cluster_storage_c = cluster_storage.clone();
        let storage_cluster_c = storage_cluster.clone();
        let broker_cluster_c = broker_cluster.clone();
        let tcp_thread_join = tcp_thread.spawn(move || {
            let runtime = create_runtime("meta-tcp-runtime", config.runtime_work_threads);

            let cf1 = config.clone();
            let cls1 = cluster_clone.clone();
            let rocksdb_storage_c1 = rocksdb_storage_c.clone();
            let cluster_storage_c1 = cluster_storage_c.clone();
            runtime.spawn(async move {
                let http_s = HttpServer::new(cf1, cls1, rocksdb_storage_c1, cluster_storage_c1);
                http_s.start().await;
            });

            let cf2 = config.clone();
            runtime.spawn(async move {
                let ip = format!("{}:{}", cf2.addr, cf2.port).parse().unwrap();

                info_meta(&format!(
                    "RobustMQ Meta Grpc Server start success. bind addr:{}",
                    ip
                ));

                let service_handler = GrpcService::new(
                    cluster_clone,
                    raft_message_send,
                    rocksdb_storage_c,
                    cluster_storage_c,
                    storage_cluster_c.clone(),
                    broker_cluster_c.clone(),
                );

                Server::builder()
                    .add_service(MetaServiceServer::new(service_handler))
                    .serve(ip)
                    .await
                    .unwrap();
            });

            runtime.block_on(async {
                if stop_recv_c.recv().await.unwrap() {
                    info_meta("TCP and GRPC Server services stop.");
                }
            });
        });
        thread_result.push(tcp_thread_join);

        // Threads that run the meta Raft engine
        let daemon_thread = thread::Builder::new().name("daemon-thread".to_owned());
        let config = self.config.clone();
        let cluster_clone = cluster.clone();
        let stop_recv_c = stop_send.subscribe();
        let rocksdb_storage_c = rocksdb_storage.clone();
        let storage_cluster_c = storage_cluster.clone();
        let broker_cluster_c = broker_cluster.clone();

        let daemon_thread_join = daemon_thread.spawn(move || {
            let daemon_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);

            daemon_runtime.spawn(async move {
                let mut peers_manager = PeersManager::new(peer_message_recv);
                peers_manager.start().await;
            });

            daemon_runtime.spawn(async move {
                loop {
                    signal::ctrl_c().await.expect("failed to listen for event");
                    match stop_send.send(true) {
                        Ok(_) => info_meta("When ctrl + c is received, the service starts to stop"),
                        Err(_) => {}
                    }
                }
            });

            // start storage engine controller;
            daemon_runtime.spawn(async move {
                let ctrl = StorageEngineController::new(storage_cluster_c);
                ctrl.start().await;
            });

            // start broker server controller;
            daemon_runtime.spawn(async move {
                let ctrl = BrokerServerController::new(broker_cluster_c);
                ctrl.start().await;
            });

            // start heartbeat check thread;
            daemon_runtime.spawn(async move {
                heartbeat.start_heartbeat_check().await;
            });

            daemon_runtime.block_on(async move {
                let mut raft: MetaRaft = MetaRaft::new(
                    config,
                    cluster_clone,
                    data_route,
                    raft_message_recv,
                    stop_recv_c,
                    rocksdb_storage_c,
                );
                raft.run().await;
            });
        });

        thread_result.push(daemon_thread_join);
        return thread_result;
    }
}
