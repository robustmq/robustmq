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

use openraft::Raft;

use crate::{core::error::MetaServiceError, raft::type_config::TypeConfig};

pub struct MultiRaftManager {
    group_num: u32,
    pub raft_group: Vec<Raft<TypeConfig>>,
}

impl MultiRaftManager {
    pub fn new(group_num: u32) -> Self {
        MultiRaftManager {
            group_num,
            raft_group: Vec::new(),
        }
    }

    pub fn start() -> Result<(), MetaServiceError> {
        Ok(())
    }

}
