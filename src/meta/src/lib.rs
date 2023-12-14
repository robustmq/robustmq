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
use self::server::GrpcService;
use common::config::meta::MetaConfig;
use common::log::{error_meta, info, info_meta};
use common::runtime::create_runtime;
use futures::executor::block_on;
use protocol::robust::meta::meta_service_server::MetaServiceServer;
use raft::election::Election;
use raft::message::RaftMessage;
use raft::node::Node;
use raft::raft::MetaRaft;
use tokio::sync::mpsc::{self, Receiver};
use tonic::transport::Server;

mod errors;
pub mod raft;
mod server;
mod storage;
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

        let meta_thread = thread::Builder::new().name("meta-thread".to_owned());
        let config = self.config.clone();
        let _ = meta_thread.spawn(move || {
            let meta_runtime = create_runtime("meta-runtime", config.runtime_work_threads);

            meta_runtime.block_on(async move {
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
            })
        });

        block_on(self.wait_meta_ready(raft_message_recv))
    }

    pub async fn wait_meta_ready(&mut self, raft_message_recv: Receiver<RaftMessage>) {
        let leader_node = self.get_leader_node().await;
        info(&format!(
            "The leader address of the cluster is {} and the node ID is {}",
            leader_node.node_ip, leader_node.node_id
        ));
        let mut meta_raft = MetaRaft::new(self.config.clone(), leader_node, raft_message_recv);
        meta_raft.run().await;
    }

    async fn get_leader_node(&self) -> Node {
        
        let mata_nodes = self.config.meta_nodes.clone();
        if mata_nodes.len() == 1 && self.node.leader_id == None {
            return Node::new(self.config.addr.clone(), self.config.node_id.clone());
        }

        // Leader Election
        let elec = Election::new(mata_nodes);
        let ld = match elec.leader_election().await {
            Ok(nd) => nd,
            Err(err) => {
                error_meta(&format!(
                    "When a node fails to obtain the Leader from another node during startup, 
                the current node is set to the Leader node. Error message {}",
                    err
                ));

                // todo We need to deal with the split-brain problem here. We'll deal with it later
                return Node::new(self.config.addr.clone(), self.config.node_id.clone());
            }
        };

        info_meta(&format!("cluster Leader is {}", ld));
        return ld;
    }
}
