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
use cluster::Cluster;
use common::config::meta::MetaConfig;
use common::log::{info, info_meta};
use common::runtime::create_runtime;
use http::server::HttpServer;
use protocol::robust::meta::meta_service_server::MetaServiceServer;
use raft::message::RaftMessage;
use raft::peer::{PeerMessage, PeersManager};
use raft::raft::MetaRaft;
use std::fmt;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use std::thread::{self, sleep};
use std::time::Duration;
use storage::route::DataRoute;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc;
use tonic::transport::Server;

pub mod broker;
pub mod client;
pub mod cluster;
mod errors;
pub mod http;
pub mod raft;
mod services;
pub mod storage;
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

pub struct Meta {
    config: MetaConfig,
}

impl Meta {
    pub fn new(config: MetaConfig) -> Meta {
        return Meta { config };
    }

    pub fn start(&mut self) {
        info(&format!(
            "Meta node starting, node IP :{}, node ID:{}",
            self.config.addr, self.config.node_id
        ));

        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
        let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);

        let cluster = Arc::new(RwLock::new(Cluster::new(
            Node::new(
                self.config.addr.clone(),
                self.config.node_id.clone(),
                self.config.port.clone(),
            ),
            peer_message_send.clone(),
        )));

        let storage = Arc::new(RwLock::new(DataRoute::new()));
        let mut thread_handles = Vec::new();

        // Thread running meta tcp server
        let tcp_thread = thread::Builder::new().name("meta-tcp-thread".to_owned());
        let config = self.config.clone();
        let cluster_clone = cluster.clone();
        let tcp_thread_join = tcp_thread.spawn(move || {
            let meta_http_runtime = create_runtime("meta-tcp-runtime", config.runtime_work_threads);

            let cf1 = config.clone();
            let cls1 = cluster_clone.clone();
            meta_http_runtime.spawn(async move {
                let http_s = HttpServer::new(cf1, cls1);
                http_s.start().await;
            });

            let cf2 = config.clone();
            meta_http_runtime.spawn(async move {
                let ip = format!("{}:{}", cf2.addr, cf2.port).parse().unwrap();

                info_meta(&format!(
                    "RobustMQ Meta Grpc Server start success. bind addr:{}",
                    ip
                ));

                let service_handler = GrpcService::new(cluster_clone, raft_message_send);

                Server::builder()
                    .add_service(MetaServiceServer::new(service_handler))
                    .serve(ip)
                    .await
                    .unwrap();
            });

            sleep(Duration::from_secs(100000000));
        });
        sleep(Duration::from_secs(2));
        thread_handles.push(tcp_thread_join);

        // Threads that run the meta Raft engine
        let datemon_thread = thread::Builder::new().name("daemon-raft-thread".to_owned());
        let config = self.config.clone();
        let cluster_clone = cluster.clone();
        let raft_thread_join = datemon_thread.spawn(move || {
            let meta_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);

            meta_runtime.spawn(async {
                let mut peers_manager = PeersManager::new(peer_message_recv);
                peers_manager.start().await;
            });

            meta_runtime.block_on(async {
                let mut raft = MetaRaft::new(config, cluster_clone, storage, raft_message_recv);
                raft.ready().await;
            });
        });
        thread_handles.push(raft_thread_join);

        thread_handles.into_iter().for_each(|handle| {
            // join() might panic in case the thread panics
            // we just ignore it
            let _ = handle.unwrap().join();
        });
    }

    pub fn watch_stop_signal(&self, stop_service_send: Sender<bool>) {
        let _ = stop_service_send.send(true);
    }
}
