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

use common_base::utils::file_utils::test_temp_dir;
use common_config::broker::{default_broker_config, default_rocksdb_family};

use crate::rocksdb::RocksDBEngine;

pub fn test_rocksdb_instance() -> Arc<RocksDBEngine> {
    let config = default_broker_config();
    Arc::new(RocksDBEngine::new(
        &test_temp_dir(),
        config.rocksdb.max_open_files,
        vec![default_rocksdb_family()],
    ))
}
