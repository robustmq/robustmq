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
use prost::Message as _;
use protocol::meta::meta_service_kv::{DeleteRequest, SetRequest};

use crate::core::error::MetaServiceError;
use crate::storage::placement::kv::KvStorage;
use rocksdb_engine::rocksdb::RocksDBEngine;

#[derive(Debug, Clone)]
pub struct DataRouteKv {
    kv_storage: KvStorage,
}

impl DataRouteKv {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
        DataRouteKv { kv_storage }
    }
    pub fn set(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req: SetRequest = SetRequest::decode(value.as_ref())?;
        Ok(self.kv_storage.set(req.key, req.value)?)
    }

    pub fn delete(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req: DeleteRequest = DeleteRequest::decode(value.as_ref())?;
        Ok(self.kv_storage.delete(req.key)?)
    }
}
