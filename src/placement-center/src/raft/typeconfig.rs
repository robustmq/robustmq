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

use std::io::Cursor;

use serde::{Deserialize, Serialize};

use crate::raft::raft_node::Node;
use crate::raft::route::AppResponseData;
use crate::route::data::StorageData;

pub type SnapshotData = Cursor<Vec<u8>>;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TypeConfig {}

impl openraft::RaftTypeConfig for TypeConfig {
    type D = StorageData;
    type R = AppResponseData;
    type Node = Node;
    type NodeId = u64;
    type Entry = openraft::impls::Entry<Self>;
    type SnapshotData = std::io::Cursor<Vec<u8>>;
    type Responder = openraft::impls::OneshotResponder<Self>;
    type AsyncRuntime = openraft::impls::TokioRuntime;
}