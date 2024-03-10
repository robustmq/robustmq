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
use cache::placement_cluster::{Node, PlacementClusterCache};
use clients::ClientPool;
use common::config::placement_center::placement_center_conf;
use common::log::info_meta;
use common::runtime::create_runtime;
use controller::broker_controller::BrokerServerController;
use controller::engine_controller::StorageEngineController;
use http::server::{start_http_server, HttpServerState};
use protocol::placement_center::placement::placement_center_service_server::PlacementCenterServiceServer;
use raft::data_route::DataRoute;
use raft::status_machine::RaftMachine;
use raft::storage::{PlacementCenterStorage, RaftMessage};
use rocksdb::raft::RaftMachineStorage;
use rocksdb::rocksdb::RocksDBEngine;
use server::grpc::GrpcService;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Mutex};
use tonic::transport::Server;
mod cache;
mod controller;
mod http;
mod raft;
mod rocksdb;
mod server;

pub struct PlacementCenter {
    server_runtime: Arc<Runtime>,
    daemon_runtime: Arc<Runtime>,
    // Cache metadata information for the Storage Engine cluster
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    // Cache metadata information for the Broker Server cluster
    broker_cache: Arc<RwLock<BrokerClusterCache>>,
    // Cache metadata information for the Placement Cluster cluster
    placement_cache: Arc<RwLock<PlacementClusterCache>>,
    // Global implementation of Raft state machine data storage
    raft_machine_storage: Arc<RwLock<RaftMachineStorage>>,
    // Raft Global read and write pointer
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    client_poll: Arc<Mutex<ClientPool>>,
}

impl PlacementCenter {
    pub fn new() -> PlacementCenter {
        let config = placement_center_conf();
        let server_runtime = Arc::new(create_runtime(
            "server-runtime",
            config.runtime_work_threads,
        ));
        let daemon_runtime = Arc::new(create_runtime(
            "daemon-runtime",
            config.runtime_work_threads,
        ));

        let engine_cache = Arc::new(RwLock::new(EngineClusterCache::new()));
        let broker_cache = Arc::new(RwLock::new(BrokerClusterCache::new()));
        let placement_cache = Arc::new(RwLock::new(PlacementClusterCache::new(
            Node::new(config.addr.clone(), config.node_id, config.grpc_port),
            config.nodes.clone(),
        )));

        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&config));

        let raft_machine_storage = Arc::new(RwLock::new(RaftMachineStorage::new(
            rocksdb_engine_handler.clone(),
        )));

        let client_poll = Arc::new(Mutex::new(ClientPool::new()));

        return PlacementCenter {
            server_runtime,
            daemon_runtime,
            engine_cache,
            broker_cache,
            placement_cache,
            raft_machine_storage,
            rocksdb_engine_handler,
            client_poll,
        };
    }

    pub fn start(&mut self, stop_send: broadcast::Sender<bool>) {
        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
        let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);
        let placement_center_storage = Arc::new(PlacementCenterStorage::new(raft_message_send));

        self.start_broker_controller();

        self.start_engine_controller(placement_center_storage.clone(), stop_send.clone());

        self.start_peers_manager(peer_message_recv);

        self.start_raft_machine(peer_message_send, raft_message_recv, stop_send.subscribe());

        self.start_http_server();

        self.start_grpc_server(placement_center_storage.clone());

        self.awaiting_stop(stop_send);
    }

    // Start HTTP Server
    pub fn start_http_server(&self) {
        let state: HttpServerState = HttpServerState::new(
            self.placement_cache.clone(),
            self.raft_machine_storage.clone(),
            self.engine_cache.clone(),
        );
        self.server_runtime.spawn(async move {
            start_http_server(state).await;
        });
    }

    // Start Grpc Server
    pub fn start_grpc_server(&self, placement_center_storage: Arc<PlacementCenterStorage>) {
        let config = placement_center_conf();
        let ip = format!("0.0.0.0:{}", config.grpc_port).parse().unwrap();
        let service_handler = GrpcService::new(
            placement_center_storage,
            self.placement_cache.clone(),
            self.engine_cache.clone(),
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
    pub fn start_engine_controller(
        &self,
        placement_center_storage: Arc<PlacementCenterStorage>,
        stop_send: broadcast::Sender<bool>,
    ) {
        let ctrl = StorageEngineController::new(
            self.engine_cache.clone(),
            placement_center_storage,
            self.rocksdb_engine_handler.clone(),
            stop_send,
        );
        self.daemon_runtime.spawn(async move {
            ctrl.start().await;
        });
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

    // Start Raft Status Machine
    pub fn start_raft_machine(
        &self,
        peer_message_send: Sender<PeerMessage>,
        raft_message_recv: Receiver<RaftMessage>,
        stop_recv: broadcast::Receiver<bool>,
    ) {
        let data_route = Arc::new(RwLock::new(DataRoute::new(
            self.rocksdb_engine_handler.clone(),
            self.engine_cache.clone(),
            self.broker_cache.clone(),
        )));

        let mut raft: RaftMachine = RaftMachine::new(
            self.placement_cache.clone(),
            data_route,
            peer_message_send,
            raft_message_recv,
            stop_recv,
            self.raft_machine_storage.clone(),
        );
        self.daemon_runtime.spawn(async move {
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
        // Wait for the stop signal
        self.daemon_runtime.block_on(async move {
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

        // todo tokio runtime shutdown
    }
}
