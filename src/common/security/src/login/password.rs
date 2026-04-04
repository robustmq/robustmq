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

use crate::manager::SecurityManager;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub fn password_check_by_login(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    username: &str,
    password: &str,
) -> bool {
    if let Some(tenant_map) = security_manager.metadata.user_info.get(tenant) {
        if let Some(user) = tenant_map.get(username) {
            return if let Some(salt) = &user.salt {
                let hash = Sha256::digest(format!("{}{}", salt, password).as_bytes());
                let hex_hash: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
                user.password == hex_hash
            } else {
                user.password == password
            };
        }
    }
    false
}
