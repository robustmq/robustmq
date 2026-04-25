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
use crate::storage::mq9::mail::Mq9MailStorage;
use bytes::Bytes;
use metadata_struct::mq9::mail::MQ9Mail;
use prost::Message as _;
use protocol::meta::meta_service_mq9::{CreateMailRequest, DeleteMailRequest};
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

    pub fn create_mail(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateMailRequest::decode(value.as_ref())?;
        let mail = MQ9Mail::decode(&req.content)?;
        let storage = Mq9MailStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&mail)?;
        Ok(())
    }

    pub fn delete_mail(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteMailRequest::decode(value.as_ref())?;
        let storage = Mq9MailStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.mail_address)?;
        Ok(())
    }
}
