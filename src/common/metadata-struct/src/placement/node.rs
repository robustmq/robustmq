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

use protocol::placement_center::placement_center_inner::ClusterType;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub cluster_name: String,
    pub cluster_type: String,
    pub create_time: u128,
    pub extend: String,
    pub node_id: u64,
    pub node_inner_addr: String,
    pub node_ip: String,
}

impl BrokerNode {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

pub fn str_to_cluster_type(cluster_type: &str) -> Option<ClusterType> {
    if *cluster_type == *ClusterType::AmqpBrokerServer.as_str_name() {
        return Some(ClusterType::AmqpBrokerServer);
    }

    if *cluster_type == *ClusterType::PlacementCenter.as_str_name() {
        return Some(ClusterType::PlacementCenter);
    }

    if *cluster_type == *ClusterType::JournalServer.as_str_name() {
        return Some(ClusterType::JournalServer);
    }

    if *cluster_type == *ClusterType::MqttBrokerServer.as_str_name() {
        return Some(ClusterType::MqttBrokerServer);
    }

    None
}
