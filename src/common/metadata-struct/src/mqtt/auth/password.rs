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

use super::storage::StorageConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PasswordBasedConfig {
    pub password_config: PasswordConfig,
    pub storage_config: StorageConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordConfig {
    pub credential_type: String, // username/clientid (only for placement)
    pub algorithm: String,       // plain/md5/sha/sha256/sha512/bcrypt/pbkdf2
    pub salt_position: Option<String>, // disable/prefix/suffix
    pub salt_rounds: Option<u32>, // bcrypt need
    pub mac_fun: Option<String>, // pbkdf2 need
    pub iterations: Option<u32>, // pbkdf2 need
    pub dk_length: Option<u32>,  // pbkdf2 need
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            credential_type: "username".to_string(),
            algorithm: "plain".to_string(),
            salt_position: Some("disable".to_string()),
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        }
    }
}
