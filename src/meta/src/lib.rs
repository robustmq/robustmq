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
use common::log::info_meta;
use common::runtime::create_runtime;
use futures::executor::block_on;
use protocol::robust::meta::meta_service_server::MetaServiceServer;
use raft::election::Election;
use raft::node::Node;
use tonic::transport::Server;
mod errors;
mod raft;
mod server;
mod storage;

pub struct Meta {
    config: MetaConfig,
    node: Node,
}

impl Meta {
    pub fn new(config: MetaConfig) -> Meta {
        let node = Node::new(config.addr.clone(), config.node_id.clone());
        return Meta { config, node };
    }

    pub fn start(&self) {
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

                let service_handler = GrpcService::new(node_state);
                Server::builder()
                    .add_service(MetaServiceServer::new(service_handler))
                    .serve(ip)
                    .await
                    .unwrap();
            })
        });
    }

    pub async fn wait_meta_ready(&self) {
        let config = self.config.meta_nodes.clone();
        // Leader Election
        let elec = Election::new(config);
        let ld = elec.leader_election().await.unwrap();
        info_meta(&format!("cluster Leader is {}", ld))
    }
}
