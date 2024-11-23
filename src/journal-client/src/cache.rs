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

use common_base::error::common::CommonError;
use dashmap::DashMap;

pub struct MetadataCache {
    shards: DashMap<String, String>,
}

impl MetadataCache {
    pub fn new() -> Self {
        let shards = DashMap::with_capacity(8);
        MetadataCache { shards }
    }
}

pub fn load_metadata_cache() {}

fn load_shards_cahce() -> Result<(), CommonError> {
    Ok(())
}

fn load_node_cache() -> Result<(), CommonError> {
    Ok(())
}
