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

use bytes::Bytes;
use common_base::error::common::CommonError;
use openraft::SnapshotMeta;
use serde::{Deserialize, Serialize};

use crate::raft::type_config::TypeConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredSnapshot {
    pub meta: SnapshotMeta<TypeConfig>,

    /// The data of the state machine at the time of this snapshot.
    pub data: Bytes,
}

pub fn build_snapshot() -> Bytes {
    Bytes::new()
}

pub fn recover_snapshot(_data: Bytes) -> Result<(), CommonError> {
    Ok(())
}
