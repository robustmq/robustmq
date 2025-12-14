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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::common::call::{get_offset_data, save_offset_data};
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_common::{
    GetOffsetDataRequest, SaveOffsetData, SaveOffsetDataRequest, SaveOffsetDataRequestOffset,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::storage::ShardOffset;

#[derive(Clone)]
pub struct OffsetStorageManager {
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
}

impl OffsetStorageManager {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        let conf = broker_config();
        OffsetStorageManager {
            client_pool,
            addrs: conf.get_meta_service_addr(),
        }
    }

    pub async fn get_offset(&self, group: &str) -> Result<Vec<ShardOffset>, CommonError> {
        let request = GetOffsetDataRequest {
            group: group.to_owned(),
        };
        let reply = get_offset_data(&self.client_pool, &self.addrs, request).await?;

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
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let offsets = offset
            .iter()
            .map(|(key, value)| SaveOffsetDataRequestOffset {
                namespace: "".to_string(),
                shard_name: key.to_string(),
                offset: *value,
            })
            .collect();

        let request = SaveOffsetDataRequest {
            offsets: vec![SaveOffsetData {
                group: group_name.to_string(),
                offsets,
            }],
        };
        save_offset_data(&self.client_pool, &self.addrs, request).await?;
        Ok(())
    }

    pub async fn batch_commit_offset(
        &self,
        offset_datas: &DashMap<String, Vec<ShardOffset>>,
    ) -> Result<(), CommonError> {
        let mut offsets = Vec::new();
        for data in offset_datas.iter() {
            let val = data
                .value()
                .iter()
                .map(|shard_offset| SaveOffsetDataRequestOffset {
                    namespace: "".to_string(),
                    shard_name: shard_offset.shard_name.to_string(),
                    offset: shard_offset.offset,
                })
                .collect();
            offsets.push(SaveOffsetData {
                group: data.key().to_string(),
                offsets: val,
            });
        }

        let request = SaveOffsetDataRequest { offsets };
        save_offset_data(&self.client_pool, &self.addrs, request).await?;
        Ok(())
    }
}
