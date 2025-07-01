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

use core::time;

use common_base::utils::time_utils;
use protocol::{
    broker_mqtt::broker_mqtt_admin::BrokerNodeRaw,
    placement_center::placement_center_inner::ClusterType,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub cluster_name: String,
    pub cluster_type: String,
    pub extend: String,
    pub node_id: u64,
    pub node_ip: String,
    pub node_inner_addr: String,
    pub start_time: u64,
    pub register_time: u64,
}

impl BrokerNode {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl From<BrokerNode> for BrokerNodeRaw {
    fn from(node: BrokerNode) -> Self {
        Self {
            cluster_name: node.cluster_name,
            cluster_type: node.cluster_type,
            extend_info: node.extend,
            node_id: node.node_id,
            node_ip: node.node_ip,
            node_inner_addr: node.node_inner_addr,
            start_time: time_utils::timestamp_to_local_datetime(node.start_time as i64),
            register_time: time_utils::timestamp_to_local_datetime(node.register_time as i64),
        }
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
