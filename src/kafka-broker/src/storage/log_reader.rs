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
use storage_adapter::storage::ArcStorageAdapter;

#[derive(Clone, Default)]
pub struct Reader {
    storage_adapter: ArcStorageAdapter,
}

impl Reader {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        Reader {
            storage_adapter
        }
    }

    async fn read_logs(
        &self,
        topic: &str,
        partition: i32,
        start_offset: i64,
        max_size: usize,
    ) -> Result<Vec<u64>, CommonError> {
        // Implementation for reading logs from storage
        Ok(vec![])
    }

}