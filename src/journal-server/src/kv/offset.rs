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

use super::engine::KvEngine;

pub struct OffsetManager {
    pub kv_engine: Arc<KvEngine>,
}

impl OffsetManager {
    pub fn new(kv_engine: Arc<KvEngine>) -> Self {
        OffsetManager { kv_engine }
    }

    pub fn commit(&self, namespace: &str, group_name: &str, shard_name: &str, offset: u64) {}

    pub fn get_group_offset(&self) {}

    pub fn get_group_shard_offset(&self) {}

    fn shard_offset_key(&self, namespace: &str, group_name: &str, shard_name: &str) -> String {
        format!(
            "{}/{}",
            self.group_offfset_key(namespace, group_name),
            shard_name
        )
    }

    fn group_offfset_key(&self, namespace: &str, group_name: &str) -> String {
        format!("/offsets/{}/{}", namespace, group_name)
    }
}
