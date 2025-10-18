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

use metadata_struct::meta::{node::BrokerNode, status::MetaStatus};
use serde::{Deserialize, Serialize};

pub mod journal;
pub mod meta;
pub mod mqtt;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PageReplyData<T> {
    pub data: T,
    pub total_count: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClusterInfoResp {
    pub version: String,
    pub cluster_name: String,
    pub start_time: u64,
    pub broker_node_list: Vec<BrokerNode>,
    pub meta: MetaStatus,
}
