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

use openraft::StorageError;

use crate::raft::route::data::StorageData;
use crate::raft::route::AppResponseData;
use std::fmt::Display;

pub type SnapshotData = tokio::fs::File;

pub type Entry = openraft::Entry<TypeConfig>;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = StorageData,
        R = AppResponseData,
        Node = Node,
        SnapshotData = SnapshotData
);

pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub node_id: u64,
    pub rpc_addr: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ rpc_addr: {}, node_id: {} }}",
            self.rpc_addr, self.node_id
        )
    }
}

pub type StorageResult<T> = Result<T, StorageError<TypeConfig>>;
