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

use std::path::Path;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use log::LogStore;
use openraft::{SnapshotMeta, StorageError};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use state::StateMachineStore;

use super::type_config::TypeConfig;
use crate::raft::route::DataRoute;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredSnapshot {
    pub meta: SnapshotMeta<TypeConfig>,

    /// The data of the state machine at the time of this snapshot.
    pub data: Bytes,
}

type StorageResult<T> = Result<T, StorageError<TypeConfig>>;

pub mod log;
pub mod snapshot;
pub mod state;

/// converts an id to a byte vector for storing in the database.
/// Note that we're using big endian encoding to ensure correct sorting of keys
fn id_to_bin(id: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8);
    buf.write_u64::<BigEndian>(id).unwrap();
    buf
}

fn bin_to_id(buf: &[u8]) -> u64 {
    (&buf[0..8]).read_u64::<BigEndian>().unwrap()
}

pub(crate) async fn new_storage<P: AsRef<Path>>(
    db_path: P,
    route: Arc<DataRoute>,
) -> (LogStore, StateMachineStore) {
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);

    let store = ColumnFamilyDescriptor::new(cf_raft_store(), Options::default());
    let logs = ColumnFamilyDescriptor::new(cf_raft_logs(), Options::default());

    let db = DB::open_cf_descriptors(&db_opts, db_path, vec![store, logs]).unwrap();
    let db = Arc::new(db);

    let log_store = LogStore { db: db.clone() };
    let sm_store = StateMachineStore::new(db, route).await.unwrap();

    (log_store, sm_store)
}

fn cf_raft_store() -> String {
    "store".to_string()
}

fn cf_raft_logs() -> String {
    "logs".to_string()
}
