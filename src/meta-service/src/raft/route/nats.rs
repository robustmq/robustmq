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

use crate::core::error::MetaServiceError;
use crate::storage::nats::NatsSubscribeStorage;
use bytes::Bytes;
use metadata_struct::nats::subscribe::NatsSubscribe;
use prost::Message as _;
use protocol::meta::meta_service_nats::{CreateNatsSubscribeRequest, DeleteNatsSubscribeRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataRouteNats {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl DataRouteNats {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        DataRouteNats {
            rocksdb_engine_handler,
        }
    }

    pub fn set_subscribe(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateNatsSubscribeRequest::decode(value.as_ref())?;
        let storage = NatsSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        for item in &req.subscribes {
            let subscribe = NatsSubscribe::decode(item)?;
            storage.save(&subscribe)?;
        }
        Ok(())
    }

    pub fn delete_subscribe(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteNatsSubscribeRequest::decode(value.as_ref())?;
        let storage = NatsSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        for key in &req.keys {
            storage.delete(&req.tenant, key.connect_id, &key.sid)?;
        }
        Ok(())
    }
}
