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

use std::sync::Arc;

use grpc_clients::pool::ClientPool;
use openraft::RaftNetworkFactory;

use super::connection::NetworkConnection;
use crate::raft::type_config::{Node, NodeId, TypeConfig};

pub struct Network {
    client_pool: Arc<ClientPool>,
    machine: String,
}

impl Network {
    pub fn new(machine: String, client_pool: Arc<ClientPool>) -> Network {
        Network {
            client_pool,
            machine,
        }
    }
}

// NOTE: This could be implemented also on `Arc<ExampleNetwork>`, but since it's empty, implemented
// directly.
impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn new_client(&mut self, _: NodeId, node: &Node) -> Self::Network {
        let addr = node.rpc_addr.to_string();
        NetworkConnection::new(self.machine.clone(), addr, self.client_pool.clone())
    }
}
