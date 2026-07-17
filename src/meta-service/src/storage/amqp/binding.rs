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

use common_base::error::common::CommonError;
use metadata_struct::amqp::binding::AmqpBinding;
use rocksdb_engine::keys::meta::{
    storage_key_amqp_binding, storage_key_amqp_binding_cluster_prefix,
    storage_key_amqp_binding_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct AmqpBindingStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AmqpBindingStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AmqpBindingStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, binding: AmqpBinding) -> Result<(), CommonError> {
        let key = storage_key_amqp_binding(&binding.tenant, &binding.key());
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, binding)
    }

    pub fn get(&self, tenant: &str, binding_key: &str) -> Result<Option<AmqpBinding>, CommonError> {
        let key = storage_key_amqp_binding(tenant, binding_key);
        Ok(
            engine_get_by_meta_metadata::<AmqpBinding>(&self.rocksdb_engine_handler, &key)?
                .map(|raw| raw.data),
        )
    }

    pub fn list_all(&self) -> Result<Vec<AmqpBinding>, CommonError> {
        let prefix_key = storage_key_amqp_binding_cluster_prefix();
        let data = engine_prefix_list_by_meta_metadata::<AmqpBinding>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<AmqpBinding>, CommonError> {
        let prefix_key = storage_key_amqp_binding_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<AmqpBinding>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, binding_key: &str) -> Result<(), CommonError> {
        let key = storage_key_amqp_binding(tenant, binding_key);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::amqp::binding::AmqpBindingDestinationType;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;

    fn setup_storage() -> AmqpBindingStorage {
        AmqpBindingStorage::new(test_rocksdb_instance())
    }

    fn create_binding(
        tenant: &str,
        source: &str,
        destination: &str,
        routing_key: &str,
    ) -> AmqpBinding {
        AmqpBinding::new(
            tenant,
            source,
            destination,
            AmqpBindingDestinationType::Queue,
            routing_key,
            HashMap::new(),
        )
    }

    #[test]
    fn test_binding_crud() {
        let storage = setup_storage();

        let b1 = create_binding("t1", "order.exchange", "order.queue", "order.created");
        storage.save(b1.clone()).unwrap();
        assert!(storage.get("t1", &b1.key()).unwrap().is_some());

        let b2 = create_binding("t1", "order.exchange", "audit.queue", "order.created");
        storage.save(b2).unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);

        let b3 = create_binding("t2", "order.exchange", "order.queue", "order.created");
        storage.save(b3).unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);
        assert_eq!(storage.list_all().unwrap().len(), 3);

        storage.delete("t1", &b1.key()).unwrap();
        assert!(storage.get("t1", &b1.key()).unwrap().is_none());
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("t1", "nonexistent").unwrap().is_none());
    }
}
