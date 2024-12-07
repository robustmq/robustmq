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

use std::collections::HashMap;
use std::sync::Arc;

use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;

use crate::storage::ShardOffset;

#[derive(Clone)]
pub(crate) struct PlaceOffsetManager {
    client_pool: Arc<ClientPool>,
}

impl PlaceOffsetManager {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        PlaceOffsetManager { client_pool }
    }

    pub async fn get_shard_offset(
        &self,
        _group: &str,
        _namespace: &str,
        _shard_names: &[String],
    ) -> Result<Vec<ShardOffset>, CommonError> {
        Ok(Vec::new())
    }

    pub async fn commit_offset(
        &self,
        _group_name: &str,
        _namespace: &str,
        _offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        Ok(())
    }
}
