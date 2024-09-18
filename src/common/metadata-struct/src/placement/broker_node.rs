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

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub cluster_name: String,
    pub cluster_type: String,
    pub node_id: u64,
    pub node_ip: String,
    pub node_inner_addr: String,
    pub extend: String,
    pub create_time: u128,
}

impl BrokerNode {
    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}