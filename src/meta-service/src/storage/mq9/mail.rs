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
use metadata_struct::mq9::email::MQ9Email;
use rocksdb_engine::keys::meta::{
    storage_key_mq9_email, storage_key_mq9_email_prefix, storage_key_mq9_email_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};
use std::sync::Arc;

pub struct Mq9EmailStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl Mq9EmailStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        Mq9EmailStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, email: &MQ9Email) -> Result<(), CommonError> {
        let key = storage_key_mq9_email(&email.tenant, &email.mail_address);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, email)
    }

    pub fn get(&self, tenant: &str, mail_address: &str) -> Result<Option<MQ9Email>, CommonError> {
        let key = storage_key_mq9_email(tenant, mail_address);
        Ok(
            engine_get_by_meta_data::<MQ9Email>(&self.rocksdb_engine_handler, &key)?
                .map(|data| data.data),
        )
    }

    pub fn list(&self) -> Result<Vec<MQ9Email>, CommonError> {
        let prefix = storage_key_mq9_email_prefix();
        let data =
            engine_prefix_list_by_meta_data::<MQ9Email>(&self.rocksdb_engine_handler, &prefix)?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<MQ9Email>, CommonError> {
        let prefix = storage_key_mq9_email_tenant_prefix(tenant);
        let data =
            engine_prefix_list_by_meta_data::<MQ9Email>(&self.rocksdb_engine_handler, &prefix)?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, mail_address: &str) -> Result<(), CommonError> {
        let key = storage_key_mq9_email(tenant, mail_address);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }
}
