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
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_RAFT;
use state::StateMachineStore;
use std::path::Path;
use std::sync::Arc;

pub mod keys;
pub mod log;
pub mod snapshot;
pub mod state;

pub(crate) async fn new_storage<P: AsRef<Path>>(
    machine: &str,
    db_path: P,
    route: Arc<DataRoute>,
) -> (LogStore, StateMachineStore) {
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);

    let store = ColumnFamilyDescriptor::new(DB_COLUMN_FAMILY_META_RAFT, Options::default());
    let db = DB::open_cf_descriptors(&db_opts, db_path, vec![store]).unwrap();
    let db = Arc::new(db);
    let log_store = LogStore {
        machine: machine.to_string(),
        db: db.clone(),
    };
    let sm_store = StateMachineStore::new(db, route).await.unwrap();

    (log_store, sm_store)
}
