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

use crate::core::cache::MQTTCacheManager;
use std::sync::Arc;

/// Check password by searching across all tenants.
/// This is intentionally tenant-agnostic because at login time the tenant
/// is not yet known (the MQTT CONNECT packet only carries username/password).
pub fn password_check_by_login(
    cache_manager: &Arc<MQTTCacheManager>,
    username: &str,
    password: &str,
) -> bool {
    for tenant_entry in cache_manager.user_info.iter() {
        if let Some(user) = tenant_entry.value().get(username) {
            return user.password == password;
        }
    }
    false
}
