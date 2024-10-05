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

use self::raftv1::peer::PeerMessage;
use crate::server::http::server::{start_http_server, HttpServerState};
use cache::journal::JournalCacheManager;
use cache::mqtt::MqttCacheManager;
use cache::placement::PlacementCacheManager;
use clients::poll::ClientPool;
use common_base::config::placement_center::placement_center_conf;
use controller::journal::controller::StorageEngineController;
use controller::mqtt::MQTTController;
use controller::placement::controller::ClusterController;
use log::info;
use openraft::Raft;
use protocol::placement_center::generate::journal::engine_service_server::EngineServiceServer;
use protocol::placement_center::generate::kv::kv_service_server::KvServiceServer;
use protocol::placement_center::generate::mqtt::mqtt_service_server::MqttServiceServer;
use protocol::placement_center::generate::placement::placement_center_service_server::PlacementCenterServiceServer;
use raftv1::apply::{RaftMachineApply, RaftMessage};
use raftv1::rocksdb::RaftMachineStorage;
use raftv2::raft_node::{create_raft_node, start_openraft_node};
use raftv2::typeconfig::TypeConfig;
use server::grpc::service_journal::GrpcEngineService;
use server::grpc::service_kv::GrpcKvService;
use server::grpc::service_mqtt::GrpcMqttService;
use server::grpc::service_placement::GrpcPlacementService;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use storage::rocksdb::{column_family_list, RocksDBEngine};
use storage::route::DataRoute;
use tokio::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};
use tokio::time::sleep;
use tonic::transport::Server;
mod cache;
mod controller;
mod core;
mod raftv1;
mod raftv2;
mod server;
mod storage;

pub struct PlacementCenter {
    // Cache metadata information for the Storage Engine cluster
    cluster_cache: Arc<PlacementCacheManager>,
    // Cache metadata information for the Broker Server cluster
    engine_cache: Arc<JournalCacheManager>,
    mqtt_cache: Arc<MqttCacheManager>,
    // Global implementation of Raft state machine data storage
    raft_machine_storage: Arc<RwLock<RaftMachineStorage>>,
    // Raft Global read and write pointer
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    // Global GRPC client connection pool
    client_poll: Arc<ClientPool>,
}

impl PlacementCenter {
    pub fn new() -> PlacementCenter {
        let config = placement_center_conf();
        let client_poll = Arc::new(ClientPool::new(100));
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));

        let engine_cache = Arc::new(JournalCacheManager::new());
        let cluster_cache: Arc<PlacementCacheManager> =
            Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let mqtt_cache: Arc<MqttCacheManager> = Arc::new(MqttCacheManager::new(
            rocksdb_engine_handler.clone(),
            cluster_cache.clone(),
        ));

        let raft_machine_storage = Arc::new(RwLock::new(RaftMachineStorage::new(
            rocksdb_engine_handler.clone(),
        )));

        return PlacementCenter {
            cluster_cache,
            engine_cache,
            mqtt_cache,
            raft_machine_storage,
            rocksdb_engine_handler,
            client_poll,
        };
    }

    pub async fn start(&mut self, stop_send: broadcast::Sender<bool>) {
        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
        let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);

        let data_route = Arc::new(DataRoute::new(
            self.rocksdb_engine_handler.clone(),
            self.cluster_cache.clone(),
            self.engine_cache.clone(),
        ));

        let openraft_node = create_raft_node(self.client_poll.clone(), data_route).await;

        let placement_center_storage = Arc::new(RaftMachineApply::new(
            raft_message_send,
            openraft_node.clone(),
        ));

        self.start_controller(placement_center_storage.clone(), stop_send.clone());

        self.start_raftv1_machine(
            peer_message_send,
            peer_message_recv,
            raft_message_recv,
            stop_send.clone(),
        );

        self.start_raftv2_machine(openraft_node.clone());

        self.start_http_server(placement_center_storage.clone());

        self.start_grpc_server(placement_center_storage.clone());

        self.awaiting_stop(stop_send).await;
    }

    // Start HTTP Server
    pub fn start_http_server(&self, raft_machine_apply: Arc<RaftMachineApply>) {
        let state: HttpServerState = HttpServerState::new(
            self.cluster_cache.clone(),
            self.raft_machine_storage.clone(),
            self.cluster_cache.clone(),
            self.engine_cache.clone(),
            raft_machine_apply.clone(),
        );
        tokio::spawn(async move {
            start_http_server(state).await;
        });
    }

    // Start Grpc Server
    pub fn start_grpc_server(&self, raft_machine_apply: Arc<RaftMachineApply>) {
        let config = placement_center_conf();
        let ip = format!("0.0.0.0:{}", config.network.grpc_port)
            .parse()
            .unwrap();
        let placement_handler = GrpcPlacementService::new(
            raft_machine_apply.clone(),
            self.cluster_cache.clone(),
            self.rocksdb_engine_handler.clone(),
            self.client_poll.clone(),
        );

        let kv_handler = GrpcKvService::new(
            raft_machine_apply.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        let engine_handler = GrpcEngineService::new(
            raft_machine_apply.clone(),
            self.cluster_cache.clone(),
            self.client_poll.clone(),
        );

        let mqtt_handler = GrpcMqttService::new(
            self.cluster_cache.clone(),
            raft_machine_apply.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        tokio::spawn(async move {
            info!("RobustMQ Meta Grpc Server start success. bind addr:{}", ip);
            Server::builder()
                .add_service(PlacementCenterServiceServer::new(placement_handler))
                .add_service(KvServiceServer::new(kv_handler))
                .add_service(MqttServiceServer::new(mqtt_handler))
                .add_service(EngineServiceServer::new(engine_handler))
                .serve(ip)
                .await
                .unwrap();
        });
    }

    // Start Storage Engine Cluster Controller
    pub fn start_controller(
        &self,
        placement_center_storage: Arc<RaftMachineApply>,
        stop_send: broadcast::Sender<bool>,
    ) {
        let ctrl = ClusterController::new(
            self.cluster_cache.clone(),
            placement_center_storage.clone(),
            stop_send.clone(),
        );
        tokio::spawn(async move {
            ctrl.start_node_heartbeat_check().await;
        });

        let mqtt_controller = MQTTController::new(
            self.rocksdb_engine_handler.clone(),
            self.cluster_cache.clone(),
            self.mqtt_cache.clone(),
            self.client_poll.clone(),
            stop_send.clone(),
        );
        tokio::spawn(async move {
            mqtt_controller.start().await;
        });

        let journal_controller = StorageEngineController::new();
        tokio::spawn(async move {
            journal_controller.start().await;
        });
    }

    // Start Raft Status Machine
    pub fn start_raftv1_machine(
        &self,
        _: Sender<PeerMessage>,
        _: Receiver<PeerMessage>,
        _: Receiver<RaftMessage>,
        _: broadcast::Sender<bool>,
    ) {
        // let data_route = Arc::new(DataRoute::new(
        //     self.rocksdb_engine_handler.clone(),
        //     self.cluster_cache.clone(),
        //     self.engine_cache.clone(),
        // ));

        // let stop_recv = stop_send.subscribe();
        // let mut raft: RaftMachine = RaftMachine::new(
        //     self.cluster_cache.clone(),
        //     data_route,
        //     peer_message_send,
        //     raft_message_recv,
        //     stop_recv,
        //     self.raft_machine_storage.clone(),
        // );
        // tokio::spawn(async move {
        //     raft.run().await;
        // });

        // let mut peers_manager =
        //     RaftPeersManager::new(peer_message_recv, self.client_poll.clone(), 5, stop_send);

        // tokio::spawn(async move {
        //     peers_manager.start().await;
        // });

        // info!("Raft Node inter-node communication management thread started successfully");
    }

    // Start Raft Status Machine
    pub fn start_raftv2_machine(&self, openraft_node: Raft<TypeConfig>) {
        tokio::spawn(async move {
            start_openraft_node(openraft_node).await;
        });
    }

    // Wait Stop Signal
    pub async fn awaiting_stop(&self, stop_send: broadcast::Sender<bool>) {
        tokio::spawn(async move {
            sleep(Duration::from_millis(5)).await;
            info!("Placement Center service started successfully...");
        });

        signal::ctrl_c().await.expect("failed to listen for event");
        match stop_send.send(true) {
            Ok(_) => {
                info!("When ctrl + c is received, the service starts to stop");
            }
            Err(_) => {}
        }
    }
}
