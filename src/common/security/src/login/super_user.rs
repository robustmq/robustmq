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
use crate::metadata::SecurityMetadata;
use crate::storage::user::UserStorage;
use common_base::error::ResultCommonError;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::tenant::DEFAULT_TENANT;
use std::sync::Arc;

pub async fn try_init_system_user(client_pool: &Arc<ClientPool>) -> ResultCommonError {
    let conf = broker_config();
    let system_user_info = SecurityUser {
        tenant: DEFAULT_TENANT.to_string(),
        username: conf.mqtt_runtime.default_user.clone(),
        password: conf.mqtt_runtime.default_password.clone(),
        salt: None,
        is_superuser: true,
        create_time: now_second(),
    };
    let user_storage = UserStorage::new(client_pool.clone());
    let res = user_storage
        .get_user(
            system_user_info.tenant.clone(),
            system_user_info.username.clone(),
        )
        .await?;
    if res.is_some() {
        return Ok(());
    }
    user_storage.save_user(system_user_info.clone()).await?;
    Ok(())
}

/// Check if a username is a super user by searching across all tenants.
pub fn is_super_user(security_manager: &Arc<SecurityManager>, username: &str) -> bool {
    if username.is_empty() {
        return false;
    }
    for tenant_entry in security_manager.security_metadata.user_info.iter() {
        if let Some(user) = tenant_entry.value().get(username) {
            return user.is_superuser;
        }
    }
    false
}
