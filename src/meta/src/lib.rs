use std::sync::{Arc, RwLock};
use std::thread;

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
use common::config::meta::MetaConfig;
use common::log::{error_meta, info, info_meta};
use common::runtime::create_runtime;
use protocol::robust::meta::meta_service_server::MetaServiceServer;
use raft::election::Election;
use raft::message::RaftMessage;
use raft::node::Node;
use raft::raft::MetaRaft;
use tokio::sync::mpsc::{self, Receiver};
use tonic::transport::Server;

mod errors;
pub mod raft;
mod services;
pub mod storage;
mod tools;

pub struct Meta {
    config: MetaConfig,
    node: Node,
}

impl Meta {
    pub fn new(config: MetaConfig) -> Meta {
        let node = Node::new(config.addr.clone(), config.node_id.clone());
        return Meta { config, node };
    }

    pub fn start(&mut self) {

        info(&format!(
            "When the node is being started, the current node IP address is {}, and the node IP address is {}",
            self.config.addr, self.config.node_id
        ));

        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(10000);

        /// Thread running meta grpc server
        let grpc_thread = thread::Builder::new().name("meta-grpc-thread".to_owned());
        let config = self.config.clone();
        let _ = grpc_thread.spawn(move || {
            let meta_http_runtime =
                create_runtime("meta-grpc-runtime", config.runtime_work_threads);
            meta_http_runtime.block_on(async move {
                let ip = format!("{}:{}", config.addr, config.port.unwrap())
                    .parse()
                    .unwrap();

                let node_state = Arc::new(RwLock::new(Node::new(config.addr, config.node_id)));

                info_meta(&format!(
                    "RobustMQ Meta Server start success. bind addr:{}",
                    ip
                ));

                let service_handler = GrpcService::new(node_state, raft_message_send);
                Server::builder()
                    .add_service(MetaServiceServer::new(service_handler))
                    .serve(ip)
                    .await
                    .unwrap();
            });
        });

        /// Threads that run the meta Raft engine
        let raft_thread = thread::Builder::new().name("meta-raft-thread".to_owned());
        let config = self.config.clone();
        let _ = raft_thread.spawn(move || {
            let meta_runtime = create_runtime("raft-runtime", 10);
            meta_runtime.block_on(async {
                let mut raft = MetaRaft::new(config, raft_message_recv);
                raft.ready().await;
            });
        });
    }
}
