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

use std::sync::Arc;

use axum::async_trait;
use clients::poll::ClientPool;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MQTTAcl;
use metadata_struct::acl::mqtt_blacklist::MQTTAclBlackList;
use metadata_struct::mqtt::user::MQTTUser;

use super::AuthStorageAdapter;
use crate::storage::acl::AclStorage;
use crate::storage::blacklist::BlackListStorage;
use crate::storage::user::UserStorage;

pub struct PlacementAuthStorageAdapter {
    client_poll: Arc<ClientPool>,
}

impl PlacementAuthStorageAdapter {
    pub fn new(client_poll: Arc<ClientPool>) -> PlacementAuthStorageAdapter {
        return PlacementAuthStorageAdapter { client_poll };
    }
}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MQTTUser>, CommonError> {
        let user_storage = UserStorage::new(self.client_poll.clone());
        return user_storage.user_list().await;
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, CommonError> {
        let user_storage = UserStorage::new(self.client_poll.clone());
        return user_storage.get_user(username).await;
    }

    async fn read_all_acl(&self) -> Result<Vec<MQTTAcl>, CommonError> {
        let acl_storage = AclStorage::new(self.client_poll.clone());
        return acl_storage.list_acl().await;
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MQTTAclBlackList>, CommonError> {
        let blacklist_storage = BlackListStorage::new(self.client_poll.clone());
        return blacklist_storage.list_blacklist().await;
    }
}
