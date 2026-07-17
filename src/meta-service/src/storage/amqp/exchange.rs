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
use metadata_struct::amqp::exchange::AmqpExchange;
use rocksdb_engine::keys::meta::{
    storage_key_amqp_exchange, storage_key_amqp_exchange_cluster_prefix,
    storage_key_amqp_exchange_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct AmqpExchangeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AmqpExchangeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AmqpExchangeStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, exchange: AmqpExchange) -> Result<(), CommonError> {
        let key = storage_key_amqp_exchange(&exchange.tenant, &exchange.exchange_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, exchange)
    }

    pub fn get(
        &self,
        tenant: &str,
        exchange_name: &str,
    ) -> Result<Option<AmqpExchange>, CommonError> {
        let key = storage_key_amqp_exchange(tenant, exchange_name);
        Ok(
            engine_get_by_meta_metadata::<AmqpExchange>(&self.rocksdb_engine_handler, &key)?
                .map(|raw| raw.data),
        )
    }

    /// All durable exchanges across every tenant — used to warm
    /// `MetaCacheManager` on startup (non-durable exchanges are never
    /// persisted here, so a restart naturally drops them).
    pub fn list_all(&self) -> Result<Vec<AmqpExchange>, CommonError> {
        let prefix_key = storage_key_amqp_exchange_cluster_prefix();
        let data = engine_prefix_list_by_meta_metadata::<AmqpExchange>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<AmqpExchange>, CommonError> {
        let prefix_key = storage_key_amqp_exchange_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<AmqpExchange>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, exchange_name: &str) -> Result<(), CommonError> {
        let key = storage_key_amqp_exchange(tenant, exchange_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::amqp::exchange::AmqpExchangeType;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;

    fn setup_storage() -> AmqpExchangeStorage {
        AmqpExchangeStorage::new(test_rocksdb_instance())
    }

    fn create_exchange(tenant: &str, name: &str) -> AmqpExchange {
        AmqpExchange::new(
            tenant,
            name,
            AmqpExchangeType::Topic,
            true,
            false,
            false,
            HashMap::new(),
        )
    }

    #[test]
    fn test_exchange_crud() {
        let storage = setup_storage();

        storage
            .save(create_exchange("t1", "order.exchange"))
            .unwrap();
        assert!(storage.get("t1", "order.exchange").unwrap().is_some());

        storage
            .save(create_exchange("t1", "audit.exchange"))
            .unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);

        storage
            .save(create_exchange("t2", "order.exchange"))
            .unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);
        assert_eq!(storage.list_by_tenant("t2").unwrap().len(), 1);

        storage.delete("t1", "audit.exchange").unwrap();
        assert!(storage.get("t1", "audit.exchange").unwrap().is_none());
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("t1", "nonexistent").unwrap().is_none());
    }
}
