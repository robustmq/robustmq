// Copyright 2023 RobustMQ Team
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


use std::sync::Arc;

use axum::async_trait;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use metadata_struct::mqtt::user::MQTTUser;

use crate::storage::user::UserStorage;

use super::AuthStorageAdapter;

pub struct PlacementAuthStorageAdapter {
    user_storage: UserStorage,
}

impl PlacementAuthStorageAdapter {
    pub fn new(client_poll: Arc<ClientPool>) -> PlacementAuthStorageAdapter {
        let user_storage = UserStorage::new(client_poll);
        return PlacementAuthStorageAdapter { user_storage };
    }
}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MQTTUser>, RobustMQError> {
        return self.user_storage.user_list().await;
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return self.user_storage.get_user(username).await;
    }
}
