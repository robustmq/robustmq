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
use crate::security::storage::storage_trait::AuthStorageAdapter;
use async_trait::async_trait;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::{enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction, tools::now_second};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::storage::RedisConfig;
use metadata_struct::mqtt::user::MqttUser;
use redis::Commands;
use std::collections::HashMap;
use third_driver::redis::{build_redis_conn_pool, RedisPool};

type RedisConnection = r2d2::PooledConnection<redis::Client>;

mod schema;
use schema::{RedisAuthAcl, RedisAuthUser};

pub struct RedisAuthStorageAdapter {
    pool: RedisPool,
    #[allow(dead_code)]
    config: RedisConfig,
}

impl RedisAuthStorageAdapter {
    pub fn new(config: RedisConfig) -> Self {
        // build Redis connection address according to configuration
        let addr = if config.password.is_empty() {
            format!("redis://{}/{}", config.redis_addr, config.database)
        } else {
            format!(
                "redis://:{}@{}/{}",
                config.password, config.redis_addr, config.database
            )
        };

        let pool = match build_redis_conn_pool(&addr) {
            Ok(data) => data,
            Err(e) => {
                panic!("Failed to create Redis connection pool: {}", e);
            }
        };
        RedisAuthStorageAdapter { pool, config }
    }
}

#[async_trait]
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
                            // todo bugfix
                            create_time: now_second(),
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
}
