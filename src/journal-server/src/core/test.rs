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

use common_base::tools::unique_id;
use rocksdb_engine::RocksDBEngine;

use crate::index::engine::{column_family_list, storage_data_fold};
use crate::segment::SegmentIdentity;

pub fn build_rs_sg() -> (Arc<RocksDBEngine>, SegmentIdentity) {
    let data_fold = vec![format!("/tmp/tests/{}", unique_id())];

    let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
        &storage_data_fold(&data_fold),
        10000,
        column_family_list(),
    ));

    let namespace = unique_id();
    let shard_name = "s1".to_string();
    let segment_no = 10;

    let segment_iden = SegmentIdentity {
        namespace: namespace.clone(),
        shard_name: shard_name.clone(),
        segment_seq: segment_no,
    };
    (rocksdb_engine_handler, segment_iden)
}
