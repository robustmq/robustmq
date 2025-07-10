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

use super::cache::CacheManager;
use crate::storage::user::UserStorage;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use std::sync::Arc;

pub async fn init_system_user(cache_manager: &Arc<CacheManager>, client_pool: &Arc<ClientPool>) {
    let conf = broker_mqtt_conf();
    let system_user_info = MqttUser {
        username: conf.system.default_user.clone(),
        password: conf.system.default_password.clone(),
        is_superuser: true,
    };
    let user_storage = UserStorage::new(client_pool.clone());
    match user_storage.save_user(system_user_info.clone()).await {
        Ok(_) => {
            cache_manager.add_user(system_user_info);
        }
        Err(e) => {
            if !e.to_string().contains("already exist") {
                panic!("{}", e.to_string());
            }
        }
    }
}
