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

use dashmap::DashMap;

pub struct Reader {
    group_local_cache_channel: DashMap<String, String>,
}

impl Reader {
    pub fn new() -> Self {
        let group_local_cache_channel = DashMap::with_capacity(2);
        Reader {
            group_local_cache_channel,
        }
    }

    pub fn read() {}
}
