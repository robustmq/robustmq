// Copyright 2023 RobustMQ Team
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
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::kv::{DeleteRequest, SetRequest};
use tonic::Status;
use crate::storage::{placement::kv::KvStorage, rocksdb::RocksDBEngine};
pub struct DataRouteKv {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    kv_storage: KvStorage,
}

impl DataRouteKv {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
        return DataRouteKv {
            rocksdb_engine_handler,
            kv_storage,
        };
    }
    pub fn set(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: SetRequest = SetRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.set(req.key, req.value);
        return Ok(());
    }

    pub fn delete(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: DeleteRequest = DeleteRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.delete(req.key);
        return Ok(());
    }
}
