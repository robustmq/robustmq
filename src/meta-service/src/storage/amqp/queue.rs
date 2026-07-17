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
use metadata_struct::amqp::queue::AmqpQueue;
use rocksdb_engine::keys::meta::{
    storage_key_amqp_queue, storage_key_amqp_queue_cluster_prefix,
    storage_key_amqp_queue_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct AmqpQueueStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AmqpQueueStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AmqpQueueStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, queue: AmqpQueue) -> Result<(), CommonError> {
        let key = storage_key_amqp_queue(&queue.tenant, &queue.queue_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, queue)
    }

    pub fn get(&self, tenant: &str, queue_name: &str) -> Result<Option<AmqpQueue>, CommonError> {
        let key = storage_key_amqp_queue(tenant, queue_name);
        Ok(
            engine_get_by_meta_metadata::<AmqpQueue>(&self.rocksdb_engine_handler, &key)?
                .map(|raw| raw.data),
        )
    }

    /// All durable queues across every tenant — used to warm `MetaCacheManager`
    /// on startup (non-durable queues are never persisted here, so a restart
    /// naturally drops them).
    pub fn list_all(&self) -> Result<Vec<AmqpQueue>, CommonError> {
        let prefix_key = storage_key_amqp_queue_cluster_prefix();
        let data = engine_prefix_list_by_meta_metadata::<AmqpQueue>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<AmqpQueue>, CommonError> {
        let prefix_key = storage_key_amqp_queue_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<AmqpQueue>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, queue_name: &str) -> Result<(), CommonError> {
        let key = storage_key_amqp_queue(tenant, queue_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;

    fn setup_storage() -> AmqpQueueStorage {
        AmqpQueueStorage::new(test_rocksdb_instance())
    }

    fn create_queue(tenant: &str, name: &str) -> AmqpQueue {
        AmqpQueue::new(tenant, name, true, false, false, HashMap::new())
    }

    #[test]
    fn test_queue_crud() {
        let storage = setup_storage();

        storage.save(create_queue("t1", "order.queue")).unwrap();
        assert!(storage.get("t1", "order.queue").unwrap().is_some());

        storage.save(create_queue("t1", "audit.queue")).unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);

        storage.save(create_queue("t2", "order.queue")).unwrap();
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 2);
        assert_eq!(storage.list_by_tenant("t2").unwrap().len(), 1);
        assert_eq!(storage.list_all().unwrap().len(), 3);

        storage.delete("t1", "audit.queue").unwrap();
        assert!(storage.get("t1", "audit.queue").unwrap().is_none());
        assert_eq!(storage.list_by_tenant("t1").unwrap().len(), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("t1", "nonexistent").unwrap().is_none());
    }
}
