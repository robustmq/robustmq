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

use crate::raft::route::DataRoute;
use log::LogStore;
use rocksdb_engine::rocksdb::RocksDBEngine;
use state::StateMachineStore;
use std::sync::Arc;

pub mod keys;
pub mod log;
pub mod snapshot;
pub mod state;

pub(crate) async fn new_storage(
    machine: &str,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    route: Arc<DataRoute>,
) -> (LogStore, StateMachineStore) {
    let log_store = LogStore {
        machine: machine.to_string(),
        db: rocksdb_engine_handler.db.clone(),
    };
    let sm_store = StateMachineStore::new(machine.to_string(), route)
        .await
        .unwrap();

    (log_store, sm_store)
}
