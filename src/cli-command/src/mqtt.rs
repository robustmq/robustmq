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

use clients::mqtt::admin::call::cluster_status;
use clients::poll::ClientPool;
use protocol::broker_server::generate::admin::ClusterStatusRequest;

use crate::{error_info, grpc_addr};

#[derive(Clone)]
pub struct MqttCliCommandParam {
    pub server: String,
    pub action: String,
}

pub enum MqttActionType {
    STATUS,
}

impl From<String> for MqttActionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "status" => MqttActionType::STATUS,
            _ => panic!("Invalid action type {}", s),
        }
    }
}

pub struct MqttBrokerCommand {}

impl Default for MqttBrokerCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttBrokerCommand {
    pub fn new() -> Self {
        MqttBrokerCommand {}
    }

    pub async fn start(&self, params: MqttCliCommandParam) {
        let action_type = MqttActionType::from(params.action.clone());
        let client_poll = Arc::new(ClientPool::new(100));
        match action_type {
            MqttActionType::STATUS => {
                self.status(client_poll.clone(), params.clone()).await;
            }
        }
    }

    async fn status(&self, client_poll: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ClusterStatusRequest {};
        match cluster_status(client_poll, grpc_addr(params.server), request).await {
            Ok(data) => {
                println!("cluster name: {}", data.cluster_name);
                println!("node list:");
                for node in data.nodes {
                    println!("- {}", node);
                }
                println!("MQTT broker cluster up and running")
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }
}
