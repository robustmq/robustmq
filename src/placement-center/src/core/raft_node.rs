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

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum RaftNodeState {
    #[default]
    Running,
    Starting,
    Stopping,
    Stop,
}

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct RaftNode {
    pub node_id: u64,
    pub node_addr: String,
}
