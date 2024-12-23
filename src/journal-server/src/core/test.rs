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

use common_base::config::journal_server::{
    init_journal_server_conf_by_config, JournalServerConfig,
};
use common_base::tools::unique_id;
use rocksdb_engine::RocksDBEngine;

use crate::index::engine::{column_family_list, storage_data_fold};
use crate::segment::SegmentIdentity;

pub fn test_build_rocksdb_sgement() -> (Arc<RocksDBEngine>, SegmentIdentity) {
    let data_fold = test_build_data_fold();
    let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
        &storage_data_fold(&data_fold),
        10000,
        column_family_list(),
    ));

    let segment_iden = test_build_segment();
    (rocksdb_engine_handler, segment_iden)
}

pub fn test_build_segment() -> SegmentIdentity {
    let namespace = unique_id();
    let shard_name = "s1".to_string();
    let segment_no = 10;

    SegmentIdentity {
        namespace: namespace.clone(),
        shard_name: shard_name.clone(),
        segment_seq: segment_no,
    }
}

pub fn test_build_data_fold() -> Vec<String> {
    let data_fold = vec![format!("/tmp/tests/{}", unique_id())];
    data_fold
}

pub fn test_init_conf() {
    init_journal_server_conf_by_config(JournalServerConfig {
        node_id: 1,
        ..Default::default()
    });
}
