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

#[derive(Default, Clone)]
pub struct JournalClientOption {
    pub addrs: Vec<String>,
    pub line_ms: u64,
}

impl JournalClientOption {
    pub fn build() -> Self {
        JournalClientOption {
            line_ms: 10,
            ..Default::default()
        }
    }

    pub fn set_addrs(&mut self, addrs: Vec<String>) {
        self.addrs = addrs;
    }
}

pub fn options_validator(option: &JournalClientOption) -> Result<(), CommonError> {
    if option.addrs.is_empty() {
        return Err(CommonError::ParameterCannotBeNull(
            "option.addrs".to_string(),
        ));
    }
    Ok(())
}
