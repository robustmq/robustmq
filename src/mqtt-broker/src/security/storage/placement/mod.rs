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

use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use crate::security::AuthStorageAdapter;
use crate::storage::acl::AclStorage;
use crate::storage::blacklist::BlackListStorage;
use crate::storage::user::UserStorage;
use axum::async_trait;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use std::sync::Arc;

pub struct PlacementAuthStorageAdapter {
    client_pool: Arc<ClientPool>,
}

impl PlacementAuthStorageAdapter {
    pub fn new(client_pool: Arc<ClientPool>) -> PlacementAuthStorageAdapter {
        PlacementAuthStorageAdapter { client_pool }
    }
}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let user_storage = UserStorage::new(self.client_pool.clone());
        return user_storage.user_list().await;
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let acl_storage = AclStorage::new(self.client_pool.clone());
        return acl_storage.list_acl().await;
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let blacklist_storage = BlackListStorage::new(self.client_pool.clone());
        return blacklist_storage.list_blacklist().await;
    }

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        let user_storage = UserStorage::new(self.client_pool.clone());
        return user_storage.get_user(username).await;
    }

    async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let user_storage = UserStorage::new(self.client_pool.clone());
        return user_storage.save_user(user_info).await;
    }

    async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        let user_storage = UserStorage::new(self.client_pool.clone());
        return user_storage.delete_user(username).await;
    }

    async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let acl_storage = AclStorage::new(self.client_pool.clone());
        return acl_storage.save_acl(acl).await;
    }

    async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let acl_storage = AclStorage::new(self.client_pool.clone());
        return acl_storage.delete_acl(acl).await;
    }

    async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        let blacklist_storage = BlackListStorage::new(self.client_pool.clone());
        blacklist_storage.save_blacklist(blacklist).await
    }

    async fn delete_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        let blacklist_storage = BlackListStorage::new(self.client_pool.clone());
        blacklist_storage.delete_blacklist(blacklist).await
    }
}
