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
use protocol::journal_server::journal_record::JournalRecord;

pub struct SegentWrite {
    last_write_time: u64,
}

pub struct WriteManager {
    segment_write: DashMap<String, SegentWrite>,
}

impl WriteManager {
    pub fn new() -> Self {
        WriteManager {
            segment_write: DashMap::with_capacity(8),
        }
    }

    pub fn write(&self, namespace: &str, shard_name: &str, segment: u32, datas: &[JournalRecord]) {}

    fn get_write(&self, namespace: &str, shard_name: &str, segment: u32) {
        let key = self.key(namespace, shard_name, segment);
        // let write = if let Some(write) = self.segment_write.get(&key) {
        //     write
        // } else {
        //     // let (sender, recv) = mpsc::channel::<JournalRecord>(1000);
        // };
        // write
    }

    fn key(&self, namespace: &str, shard_name: &str, segment: u32) -> String {
        format!("{}_{}_{}", namespace, shard_name, segment)
    }
}
