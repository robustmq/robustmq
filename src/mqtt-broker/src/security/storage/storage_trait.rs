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

use axum::async_trait;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError>;

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError>;

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError>;

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError>;

    async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError;

    async fn delete_user(&self, username: String) -> ResultMqttBrokerError;

    async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError;

    async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError;

    async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError;

    async fn delete_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError;
}
