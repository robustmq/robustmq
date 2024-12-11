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
use grpc_clients::placement::inner::call::{get_offset_data, save_offset_data};
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_inner::{
    GetOffsetDataRequest, SaveOffsetDataRequest, SaveOffsetDataRequestOffset,
};

use crate::storage::ShardOffset;

#[derive(Clone)]
pub(crate) struct PlaceOffsetManager {
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
}

impl PlaceOffsetManager {
    pub fn new(client_pool: Arc<ClientPool>, addrs: Vec<String>) -> Self {
        PlaceOffsetManager { client_pool, addrs }
    }

    pub async fn get_shard_offset(
        &self,
        cluster_name: &str,
        group: &str,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let request = GetOffsetDataRequest {
            cluster_name: cluster_name.to_owned(),
            group: group.to_owned(),
        };
        let reply = get_offset_data(self.client_pool.clone(), &self.addrs.clone(), request).await?;
        let mut results = Vec::new();
        for raw in reply.offsets {
            results.push(ShardOffset {
                shard_name: raw.shard_name,
                offset: raw.offset,
                ..Default::default()
            });
        }
        Ok(results)
    }

    pub async fn commit_offset(
        &self,
        cluster_name: &str,
        group_name: &str,
        namespace: &str,
        offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let mut offset_data = Vec::new();
        for (shard_name, offset) in offset {
            offset_data.push(SaveOffsetDataRequestOffset {
                namespace: namespace.to_owned(),
                shard_name,
                offset,
            });
        }
        let request = SaveOffsetDataRequest {
            cluster_name: cluster_name.to_owned(),
            group: group_name.to_owned(),
            offsets: offset_data,
        };
        save_offset_data(self.client_pool.clone(), &self.addrs.clone(), request).await?;
        Ok(())
    }
}
