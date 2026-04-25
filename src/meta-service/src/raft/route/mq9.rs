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
use crate::storage::mq9::email::Mq9EmailStorage;
use bytes::Bytes;
use metadata_struct::mq9::email::MQ9Email;
use prost::Message as _;
use protocol::meta::meta_service_mq9::{CreateEmailRequest, DeleteEmailRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataRouteMq9 {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl DataRouteMq9 {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        DataRouteMq9 {
            rocksdb_engine_handler,
        }
    }

    pub fn create_email(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateEmailRequest::decode(value.as_ref())?;
        let email = MQ9Email::decode(&req.content)?;
        let storage = Mq9EmailStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&email)?;
        Ok(())
    }

    pub fn delete_email(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteEmailRequest::decode(value.as_ref())?;
        let storage = Mq9EmailStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.mail_address)?;
        Ok(())
    }
}
