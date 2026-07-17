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

use bytes::Bytes;
use metadata_struct::amqp::binding::AmqpBinding;
use metadata_struct::amqp::exchange::AmqpExchange;
use metadata_struct::amqp::queue::AmqpQueue;
use prost::Message;
use protocol::meta::meta_service_amqp::{
    DeleteBindingRequest, DeleteExchangeRequest, DeleteQueueRequest, SetBindingRequest,
    SetExchangeRequest, SetQueueRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::amqp::binding::AmqpBindingStorage;
use crate::storage::amqp::exchange::AmqpExchangeStorage;
use crate::storage::amqp::queue::AmqpQueueStorage;

#[derive(Clone)]
pub struct DataRouteAmqp {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<MetaCacheManager>,
}

impl DataRouteAmqp {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<MetaCacheManager>,
    ) -> Self {
        DataRouteAmqp {
            rocksdb_engine_handler,
            cache_manager,
        }
    }

    /// Applied on every meta-service node once raft commits the entry, so this
    /// is what keeps `MetaCacheManager` in sync cluster-wide. Only durable
    /// exchanges also get persisted to rocksdb — a non-durable one is meant to
    /// disappear once every node restarts, so it must never touch disk.
    pub fn set_exchange(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetExchangeRequest::decode(value.as_ref())?;
        let exchange = AmqpExchange::decode(&req.exchange)?;
        self.cache_manager.set_exchange(exchange.clone());
        if exchange.durable {
            let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
            storage.save(exchange)?;
        }
        Ok(())
    }

    pub fn delete_exchange(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteExchangeRequest::decode(value.as_ref())?;
        self.cache_manager
            .remove_exchange(&req.tenant, &req.exchange_name);
        let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.exchange_name)?;
        Ok(())
    }

    /// Same durable/non-durable split as set_exchange. The queue's message
    /// shard (a Topic, TopicSource::AMQP) is a separate concern handled
    /// elsewhere and is always persisted regardless of this flag.
    pub fn set_queue(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetQueueRequest::decode(value.as_ref())?;
        let queue = AmqpQueue::decode(&req.queue)?;
        self.cache_manager.set_queue(queue.clone());
        if queue.durable {
            let storage = AmqpQueueStorage::new(self.rocksdb_engine_handler.clone());
            storage.save(queue)?;
        }
        Ok(())
    }

    pub fn delete_queue(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteQueueRequest::decode(value.as_ref())?;
        self.cache_manager
            .remove_queue(&req.tenant, &req.queue_name);
        let storage = AmqpQueueStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.queue_name)?;
        Ok(())
    }

    pub fn set_binding(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetBindingRequest::decode(value.as_ref())?;
        let binding = AmqpBinding::decode(&req.binding)?;
        self.cache_manager.set_binding(binding.clone());
        let storage = AmqpBindingStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(binding)?;
        Ok(())
    }

    pub fn delete_binding(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteBindingRequest::decode(value.as_ref())?;
        let key = format!(
            "{}/{}/{}/{}",
            req.source, req.destination_type, req.destination, req.routing_key
        );
        self.cache_manager.remove_binding(&req.tenant, &key);
        let storage = AmqpBindingStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &key)?;
        Ok(())
    }
}
