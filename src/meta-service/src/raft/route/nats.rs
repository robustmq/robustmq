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
use crate::storage::nats::NatsSubjectStorage;
use bytes::Bytes;
use metadata_struct::nats::subject::NatsSubject;
use prost::Message as _;
use protocol::meta::meta_service_nats::{CreateNatsSubjectRequest, DeleteNatsSubjectRequest};
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

    pub fn set_subject(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateNatsSubjectRequest::decode(value.as_ref())?;
        let subject = NatsSubject::decode(&req.subject)?;
        let storage = NatsSubjectStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&subject)?;
        Ok(())
    }

    pub fn delete_subject(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteNatsSubjectRequest::decode(value.as_ref())?;
        let storage = NatsSubjectStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.name)?;
        Ok(())
    }
}
