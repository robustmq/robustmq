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

use crate::core::error::MetaServiceError;
use crate::raft::route::data::StorageData;
use crate::raft::type_config::TypeConfig;
use openraft::raft::ClientWriteResponse;
use openraft::Raft;
use std::time::Duration;
use tokio::time::timeout;

pub struct RaftMachineManager {
    pub raft_node: Raft<TypeConfig>,
}

impl RaftMachineManager {
    pub fn new(raft_node: Raft<TypeConfig>) -> Self {
        RaftMachineManager { raft_node }
    }

    pub async fn client_write(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, MetaServiceError> {
        match self.raft_write(data).await {
            Ok(data) => Ok(Some(data)),
            Err(e) => Err(e),
        }
    }

    async fn raft_write(
        &self,
        data: StorageData,
    ) -> Result<ClientWriteResponse<TypeConfig>, MetaServiceError> {
        let resp = timeout(Duration::from_secs(10), self.raft_node.client_write(data)).await?;
        Ok(resp?)
    }
}
