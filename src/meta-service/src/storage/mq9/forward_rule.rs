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
use metadata_struct::mq9::forward_rule::Mq9ForwardRule;
use rocksdb_engine::keys::meta::{
    storage_key_mq9_forward_rule, storage_key_mq9_forward_rule_prefix,
    storage_key_mq9_forward_rule_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};
use std::sync::Arc;

pub struct Mq9ForwardRuleStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl Mq9ForwardRuleStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        Mq9ForwardRuleStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, rule: &Mq9ForwardRule) -> Result<(), CommonError> {
        let key = storage_key_mq9_forward_rule(&rule.tenant, &rule.rule_name);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, rule)
    }

    pub fn get(
        &self,
        tenant: &str,
        rule_name: &str,
    ) -> Result<Option<Mq9ForwardRule>, CommonError> {
        let key = storage_key_mq9_forward_rule(tenant, rule_name);
        Ok(
            engine_get_by_meta_data::<Mq9ForwardRule>(&self.rocksdb_engine_handler, &key)?
                .map(|data| data.data),
        )
    }

    pub fn list(&self) -> Result<Vec<Mq9ForwardRule>, CommonError> {
        let prefix = storage_key_mq9_forward_rule_prefix();
        let data = engine_prefix_list_by_meta_data::<Mq9ForwardRule>(
            &self.rocksdb_engine_handler,
            &prefix,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<Mq9ForwardRule>, CommonError> {
        let prefix = storage_key_mq9_forward_rule_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_data::<Mq9ForwardRule>(
            &self.rocksdb_engine_handler,
            &prefix,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, rule_name: &str) -> Result<(), CommonError> {
        let key = storage_key_mq9_forward_rule(tenant, rule_name);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }
}
