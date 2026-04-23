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

use common_base::error::{common::CommonError, ResultCommonError};
use metadata_struct::mqtt::share_group::ShareGroup;
use metadata_struct::mqtt::share_group::ShareGroupMember;
use rocksdb_engine::{
    keys::meta::{
        storage_key_share_group, storage_key_share_group_member,
        storage_key_share_group_member_prefix, storage_key_share_group_prefix,
        storage_key_share_group_tenant_prefix,
    },
    rocksdb::RocksDBEngine,
    storage::meta_data::{
        engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
        engine_save_by_meta_data,
    },
};
use std::{collections::HashMap, sync::Arc};

pub struct ShareGroupStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShareGroupStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShareGroupStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, group: ShareGroup) -> ResultCommonError {
        let key = storage_key_share_group(&group.tenant, &group.group_name);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, group)
    }

    pub fn get(&self, tenant: &str, group_name: &str) -> Result<Option<ShareGroup>, CommonError> {
        let key = storage_key_share_group(tenant, group_name);
        Ok(
            engine_get_by_meta_data::<ShareGroup>(&self.rocksdb_engine_handler, &key)?
                .map(|w| w.data),
        )
    }

    pub fn delete(&self, tenant: &str, group_name: &str) -> ResultCommonError {
        let key = storage_key_share_group(tenant, group_name);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<HashMap<String, ShareGroup>, CommonError> {
        let prefix_key = storage_key_share_group_tenant_prefix(tenant);
        let result = engine_prefix_list_by_meta_data::<ShareGroup>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        let mut results = HashMap::new();
        for item in result {
            results.insert(item.data.group_name.to_string(), item.data.clone());
        }
        Ok(results)
    }

    pub fn list_all(&self) -> Result<HashMap<String, ShareGroup>, CommonError> {
        let prefix_key = storage_key_share_group_prefix();
        let result = engine_prefix_list_by_meta_data::<ShareGroup>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        let mut results = HashMap::new();
        for item in result {
            results.insert(item.data.group_name.to_string(), item.data.clone());
        }
        Ok(results)
    }

    pub fn save_member(
        &self,
        tenant: &str,
        group_name: &str,
        member: &ShareGroupMember,
    ) -> ResultCommonError {
        let key =
            storage_key_share_group_member(tenant, group_name, member.broker_id, member.connect_id);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, member.clone())
    }

    pub fn get_member(
        &self,
        tenant: &str,
        group_name: &str,
        broker_id: u64,
        connect_id: u64,
    ) -> Result<Option<ShareGroupMember>, CommonError> {
        let key = storage_key_share_group_member(tenant, group_name, broker_id, connect_id);
        Ok(
            engine_get_by_meta_data::<ShareGroupMember>(&self.rocksdb_engine_handler, &key)?
                .map(|w| w.data),
        )
    }

    pub fn list_members(
        &self,
        tenant: &str,
        group_name: &str,
    ) -> Result<Vec<ShareGroupMember>, CommonError> {
        let prefix_key = storage_key_share_group_member_prefix(tenant, group_name);
        let result = engine_prefix_list_by_meta_data::<ShareGroupMember>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(result.into_iter().map(|w| w.data).collect())
    }

    pub fn delete_member(
        &self,
        tenant: &str,
        group_name: &str,
        broker_id: u64,
        connect_id: u64,
    ) -> ResultCommonError {
        let key = storage_key_share_group_member(tenant, group_name, broker_id, connect_id);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }
}
