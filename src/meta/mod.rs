use std::sync::{Arc, RwLock};

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
use self::node::Node;
use self::proto::meta::meta_service_server::MetaServiceServer;
use self::server::GrpcService;
use crate::config::meta::MetaConfig;
use crate::log;
use tokio::runtime::Runtime;
use tonic::transport::Server;
mod errors;
mod node;
mod proto;
mod raft;
mod server;
mod storage;

pub fn start(runtime: Runtime, config: MetaConfig) {
    
    let ip = format!("{}:{}", config.addr, config.port.unwrap())
        .parse()
        .unwrap();
    
    let node_state = Arc::new(RwLock::new(Node::new(config.addr, config.node_id)));
    
    runtime.spawn(async move {
        log::info(&format!("MetaSerice listening on {}", &ip));
        let service_handler = GrpcService::new(node_state);
        Server::builder()
            .add_service(MetaServiceServer::new(service_handler))
            .serve(ip)
            .await
            .unwrap();
    });
}
