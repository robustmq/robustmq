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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;
use crate::security::AuthStorageAdapter;
use axum::async_trait;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use redis::Commands;
use std::collections::HashMap;
use third_driver::redis::{build_redis_conn_pool, RedisPool};

type RedisConnection = r2d2::PooledConnection<redis::Client>;

mod schema;
use schema::{RedisAuthAcl, RedisAuthUser};

pub struct RedisAuthStorageAdapter {
    pool: RedisPool,
}

impl RedisAuthStorageAdapter {
    pub fn new(addr: String) -> Self {
        let pool = match build_redis_conn_pool(&addr) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        RedisAuthStorageAdapter { pool }
    }
}

#[async_trait]
#[allow(dependency_on_unit_never_type_fallback)]
impl AuthStorageAdapter for RedisAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let results = DashMap::with_capacity(2);

        let usernames: Vec<String> = conn.smembers(RedisAuthUser::redis_users_key())?;

        for username in usernames {
            let user_key = RedisAuthUser::redis_user_key(&username);
            let fields: HashMap<String, String> = conn.hgetall(&user_key)?;

            if !fields.is_empty() {
                match RedisAuthUser::from_redis_hash(username.clone(), fields) {
                    Ok(redis_user) => {
                        let user = MqttUser {
                            username: redis_user.username.clone(),
                            password: redis_user.password,
                            salt: if redis_user.salt.is_empty() {
                                None
                            } else {
                                Some(redis_user.salt)
                            },
                            is_superuser: redis_user.is_superuser == 1,
                        };
                        results.insert(redis_user.username, user);
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(results)
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let mut results = Vec::new();

        let acl_ids: Vec<String> = conn.smembers(RedisAuthAcl::redis_acls_key())?;

        for acl_id in acl_ids {
            let acl_key = RedisAuthAcl::redis_acl_key(&acl_id);
            let fields: HashMap<String, String> = conn.hgetall(&acl_key)?;

            if !fields.is_empty() {
                match RedisAuthAcl::from_redis_hash(acl_id, fields) {
                    Ok(redis_acl) => {
                        let acl = MqttAcl {
                            permission: match redis_acl.permission {
                                0 => MqttAclPermission::Deny,
                                1 => MqttAclPermission::Allow,
                                _ => return Err(MqttBrokerError::InvalidAclPermission),
                            },
                            resource_type: match redis_acl.username.is_empty() {
                                true => MqttAclResourceType::ClientId,
                                false => MqttAclResourceType::User,
                            },
                            resource_name: match redis_acl.username.is_empty() {
                                true => redis_acl.clientid,
                                false => redis_acl.username,
                            },
                            topic: redis_acl.topic,
                            ip: redis_acl.ipaddr,
                            action: match redis_acl.access {
                                0 => MqttAclAction::All,
                                1 => MqttAclAction::Subscribe,
                                2 => MqttAclAction::Publish,
                                3 => MqttAclAction::PubSub,
                                4 => MqttAclAction::Retain,
                                5 => MqttAclAction::Qos,
                                _ => return Err(MqttBrokerError::InvalidAclAction),
                            },
                        };
                        results.push(acl);
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(results)
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        Ok(Vec::new())
    }

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let user_key = RedisAuthUser::redis_user_key(&username);

        let fields: HashMap<String, String> = conn.hgetall(&user_key)?;

        if fields.is_empty() {
            return Ok(None);
        }

        match RedisAuthUser::from_redis_hash(username, fields) {
            Ok(redis_user) => Ok(Some(MqttUser {
                username: redis_user.username,
                password: redis_user.password,
                salt: if redis_user.salt.is_empty() {
                    None
                } else {
                    Some(redis_user.salt)
                },
                is_superuser: redis_user.is_superuser == 1,
            })),
            Err(_) => Ok(None),
        }
    }

    async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let mut conn: RedisConnection = self.pool.get()?;

        let redis_user = RedisAuthUser {
            username: user_info.username.clone(),
            password: user_info.password,
            salt: user_info.salt.unwrap_or_default(),
            is_superuser: if user_info.is_superuser { 1 } else { 0 },
        };

        let user_key = RedisAuthUser::redis_user_key(&redis_user.username);
        let hash_data = redis_user.to_redis_hash();

        for (field, value) in hash_data {
            conn.hset::<_, _, _, ()>(&user_key, &field, &value)?;
        }

        conn.sadd::<_, _, ()>(RedisAuthUser::redis_users_key(), &redis_user.username)?;

        Ok(())
    }

    async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        let mut conn: RedisConnection = self.pool.get()?;

        let user_key = RedisAuthUser::redis_user_key(&username);

        conn.del::<_, ()>(&user_key)?;

        conn.srem::<_, _, ()>(RedisAuthUser::redis_users_key(), &username)?;

        Ok(())
    }

    async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let mut conn: RedisConnection = self.pool.get()?;

        let permission: u8 = match acl.permission {
            MqttAclPermission::Allow => 1,
            MqttAclPermission::Deny => 0,
        };

        let (username, clientid) = match acl.resource_type {
            MqttAclResourceType::ClientId => (String::new(), acl.resource_name),
            MqttAclResourceType::User => (acl.resource_name, String::new()),
        };

        let access: u8 = match acl.action {
            MqttAclAction::All => 0,
            MqttAclAction::Subscribe => 1,
            MqttAclAction::Publish => 2,
            MqttAclAction::PubSub => 3,
            MqttAclAction::Retain => 4,
            MqttAclAction::Qos => 5,
        };

        let id = RedisAuthAcl::generate_id(&username, &clientid, &acl.topic);

        let redis_acl = RedisAuthAcl {
            id: id.clone(),
            username,
            permission,
            ipaddr: acl.ip,
            clientid,
            access,
            topic: acl.topic,
        };

        let acl_key = RedisAuthAcl::redis_acl_key(&redis_acl.id);
        let hash_data = redis_acl.to_redis_hash();

        for (field, value) in hash_data {
            conn.hset::<_, _, _, ()>(&acl_key, &field, &value)?;
        }

        conn.sadd::<_, _, ()>(RedisAuthAcl::redis_acls_key(), &redis_acl.id)?;

        Ok(())
    }

    async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let mut conn: RedisConnection = self.pool.get()?;

        let (username, clientid) = match acl.resource_type {
            MqttAclResourceType::ClientId => (String::new(), acl.resource_name),
            MqttAclResourceType::User => (acl.resource_name, String::new()),
        };

        let id = RedisAuthAcl::generate_id(&username, &clientid, &acl.topic);
        let acl_key = RedisAuthAcl::redis_acl_key(&id);

        conn.del::<_, ()>(&acl_key)?;

        conn.srem::<_, _, ()>(RedisAuthAcl::redis_acls_key(), &id)?;

        Ok(())
    }

    async fn save_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn delete_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn read_all_user_test() {
        let addr = "redis://127.0.0.1:6379/0".to_string();
        init_user(&addr).await;
        let auth_redis = RedisAuthStorageAdapter::new(addr);
        let result = auth_redis.read_all_user().await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.contains_key("robustmq"));
        let user = res.get("robustmq").unwrap();
        assert_eq!(user.password, "robustmq@2024");
    }

    #[tokio::test]
    #[ignore]
    async fn get_user_test() {
        let addr = "redis://127.0.0.1:6379/0".to_string();
        init_user(&addr).await;
        let auth_redis = RedisAuthStorageAdapter::new(addr);
        let username = "robustmq".to_string();
        let result = auth_redis.get_user(username).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        let user = res.unwrap();
        assert_eq!(user.password, "robustmq@2024");
    }

    async fn init_user(addr: &str) {
        let auth_redis = RedisAuthStorageAdapter::new(addr.to_string());
        let user = MqttUser {
            username: "robustmq".to_string(),
            password: "robustmq@2024".to_string(),
            salt: None,
            is_superuser: false,
        };
        let _ = auth_redis.save_user(user).await;
    }
}
