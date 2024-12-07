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

use std::sync::Arc;

use dashmap::DashMap;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::{AutoOffsetStrategy, FetchOffsetReqBody};

use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::fetch_offset;

pub struct GroupManager {
    connection_manager: Arc<ConnectionManager>,
    group_metadata: DashMap<String, DashMap<String, u64>>,
}

impl GroupManager {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        let group_metadata = DashMap::with_capacity(8);
        GroupManager {
            group_metadata,
            connection_manager,
        }
    }

    pub fn update_group_offset(
        &self,
        group_name: &str,
        namespace: &str,
        shard_name: &str,
        offset: u64,
    ) {
        if let Some(group_meta) = self.group_metadata.get_mut(group_name) {
            let key = shard_name_iden(namespace, shard_name);
            group_meta.insert(key, offset);
        } else {
            let group_meta = DashMap::with_capacity(2);
            let key = shard_name_iden(namespace, shard_name);
            group_meta.insert(key, offset);
            self.group_metadata
                .insert(group_name.to_owned(), group_meta);
        }
    }

    pub fn get_shard_offset_by_group(
        &self,
        group_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Option<u64> {
        if let Some(group_meta) = self.group_metadata.get_mut(group_name) {
            let key = shard_name_iden(namespace, shard_name);
            if let Some(offset) = group_meta.get(&key) {
                return Some(*offset);
            }
        }

        None
    }

    pub async fn init_shard_offset_by_group(
        &self,
        group_name: &str,
        namespace: &str,
        shard_name: &str,
        auto_offset_reset: AutoOffsetStrategy,
    ) -> Result<(), JournalClientError> {
        let body = FetchOffsetReqBody::default();

        let node_id = 1;
        let resp = fetch_offset(&self.connection_manager, node_id, body).await?;
        Ok(())
    }
}
