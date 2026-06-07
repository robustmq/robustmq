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

pub mod commit;
pub mod maintain;
pub mod reconcile;
pub mod replication;

use std::sync::Arc;

use broker_core::cache::NodeCacheManager;
use common_config::config::BrokerConfig;
use rocksdb_engine::test::test_rocksdb_instance;
use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
use storage_engine::core::cache::StorageCacheManager;

pub(crate) fn make_engine() -> Arc<MemoryStorageEngine> {
    let db = test_rocksdb_instance();
    let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
    let cm = Arc::new(StorageCacheManager::new(broker_cache));
    Arc::new(MemoryStorageEngine::new(db, cm, Default::default()))
}
