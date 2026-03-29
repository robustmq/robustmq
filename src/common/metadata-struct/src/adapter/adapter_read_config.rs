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

#[derive(Default, Clone)]
pub struct AdapterReadConfig {
    pub max_record_num: u64,
    pub max_size: u64,
}

impl AdapterReadConfig {
    pub fn new() -> Self {
        AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024 * 1024,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AdapterWriteRespRow {
    pub offset: u64,
    pub pkid: u64,
    pub error: Option<String>,
}

impl AdapterWriteRespRow {
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn error_info(&self) -> String {
        if let Some(err) = self.error.clone() {
            return err;
        }
        "AdapterWriteRespRow Null Error".to_string()
    }
}
