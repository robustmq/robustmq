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
use metadata_struct::amqp::exchange::AmqpExchange;
use prost::Message;
use protocol::meta::meta_service_amqp::{DeleteExchangeRequest, SetExchangeRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::error::MetaServiceError;
use crate::storage::amqp::exchange::AmqpExchangeStorage;

#[derive(Clone)]
pub struct DataRouteAmqp {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl DataRouteAmqp {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        DataRouteAmqp {
            rocksdb_engine_handler,
        }
    }

    pub fn set_exchange(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetExchangeRequest::decode(value.as_ref())?;
        let exchange = AmqpExchange::decode(&req.exchange)?;
        let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(exchange)?;
        Ok(())
    }

    pub fn delete_exchange(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteExchangeRequest::decode(value.as_ref())?;
        let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.exchange_name)?;
        Ok(())
    }
}
