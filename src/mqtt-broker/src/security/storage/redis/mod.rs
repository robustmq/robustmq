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
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::get_blacklist_type_by_str;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::{enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction, tools::now_second};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::storage::RedisConfig;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::tenant::DEFAULT_TENANT;
use redis::Commands;
use std::collections::HashMap;
use third_driver::redis::{build_redis_conn_pool, RedisPool};
use tracing::warn;

type RedisConnection = r2d2::PooledConnection<redis::Client>;

mod schema;
use schema::{RedisAuthAcl, RedisAuthBlacklist, RedisAuthUser};

pub struct RedisAuthStorageAdapter {
    pool: RedisPool,
    #[allow(dead_code)]
    config: RedisConfig,
}

impl RedisAuthStorageAdapter {
    pub fn new(config: RedisConfig) -> Result<Self, MqttBrokerError> {
        // build Redis connection address according to configuration
        let addr = if config.password.is_empty() {
            format!("redis://{}/{}", config.redis_addr, config.database)
        } else {
            format!(
                "redis://:{}@{}/{}",
                config.password, config.redis_addr, config.database
            )
        };

        let pool = build_redis_conn_pool(&addr)?;
        Ok(RedisAuthStorageAdapter { pool, config })
    }

    fn exec_query_ids(
        conn: &mut RedisConnection,
        query: &str,
        fallback_key: String,
    ) -> Result<Vec<String>, MqttBrokerError> {
        if query.trim().is_empty() {
            return Ok(conn.smembers(fallback_key)?);
        }

        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(conn.smembers(fallback_key)?);
        }

        let mut cmd = redis::cmd(parts[0]);
        for arg in parts.iter().skip(1) {
            cmd.arg(*arg);
        }
        Ok(cmd.query::<Vec<String>>(conn)?)
    }

    fn query_user_ids(&self, conn: &mut RedisConnection) -> Result<Vec<String>, MqttBrokerError> {
        Self::exec_query_ids(
            conn,
            &self.config.query_user,
            RedisAuthUser::redis_users_key(),
        )
    }

    fn query_acl_ids(&self, conn: &mut RedisConnection) -> Result<Vec<String>, MqttBrokerError> {
        Self::exec_query_ids(conn, &self.config.query_acl, RedisAuthAcl::redis_acls_key())
    }

    fn query_blacklist_ids(
        &self,
        conn: &mut RedisConnection,
    ) -> Result<Vec<String>, MqttBrokerError> {
        Self::exec_query_ids(
            conn,
            &self.config.query_blacklist,
            RedisAuthBlacklist::redis_blacklists_key(),
        )
    }
}

#[async_trait]
impl AuthStorageAdapter for RedisAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let usernames = self.query_user_ids(&mut conn)?;
        let results = DashMap::with_capacity(usernames.len());

        if usernames.is_empty() {
            return Ok(results);
        }

        let mut pipe = redis::pipe();
        for username in &usernames {
            pipe.cmd("HGETALL")
                .arg(RedisAuthUser::redis_user_key(username));
        }
        let rows: Vec<HashMap<String, String>> = pipe.query(&mut conn)?;

        for (username, fields) in usernames.into_iter().zip(rows.into_iter()) {
            if fields.is_empty() {
                continue;
            }
            match RedisAuthUser::from_redis_hash(username.clone(), fields) {
                Ok(redis_user) => {
                    let user = MqttUser {
                        tenant: DEFAULT_TENANT.to_string(),
                        username: redis_user.username.clone(),
                        password: redis_user.password,
                        salt: if redis_user.salt.is_empty() {
                            None
                        } else {
                            Some(redis_user.salt)
                        },
                        is_superuser: redis_user.is_superuser == 1,
                        create_time: redis_user.created.unwrap_or_else(now_second),
                    };
                    results.insert(redis_user.username, user);
                }
                Err(e) => {
                    warn!(username = %username, error = %e, "Parse Redis user failed, skip record");
                }
            }
        }

        Ok(results)
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let acl_ids = self.query_acl_ids(&mut conn)?;
        let mut results = Vec::with_capacity(acl_ids.len());

        if acl_ids.is_empty() {
            return Ok(results);
        }

        let mut pipe = redis::pipe();
        for acl_id in &acl_ids {
            pipe.cmd("HGETALL").arg(RedisAuthAcl::redis_acl_key(acl_id));
        }
        let rows: Vec<HashMap<String, String>> = pipe.query(&mut conn)?;

        for (acl_id, fields) in acl_ids.into_iter().zip(rows.into_iter()) {
            if fields.is_empty() {
                continue;
            }
            match RedisAuthAcl::from_redis_hash(acl_id.clone(), fields) {
                Ok(redis_acl) => {
                    let acl = MqttAcl {
                        tenant: DEFAULT_TENANT.to_string(),
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
                Err(e) => {
                    warn!(acl_id = %acl_id, error = %e, "Parse Redis ACL failed, skip record");
                }
            }
        }

        Ok(results)
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let mut conn: RedisConnection = self.pool.get()?;
        let blacklist_ids = self.query_blacklist_ids(&mut conn)?;
        let mut results = Vec::with_capacity(blacklist_ids.len());

        if blacklist_ids.is_empty() {
            return Ok(results);
        }

        let mut pipe = redis::pipe();
        for id in &blacklist_ids {
            pipe.cmd("HGETALL")
                .arg(RedisAuthBlacklist::redis_blacklist_key(id));
        }
        let rows: Vec<HashMap<String, String>> = pipe.query(&mut conn)?;

        for (id, fields) in blacklist_ids.into_iter().zip(rows.into_iter()) {
            if fields.is_empty() {
                continue;
            }
            match RedisAuthBlacklist::from_redis_hash(id.clone(), fields) {
                Ok(raw) => {
                    let blacklist = MqttAclBlackList {
                        tenant: DEFAULT_TENANT.to_string(),
                        blacklist_type: get_blacklist_type_by_str(&raw.blacklist_type)?,
                        resource_name: raw.resource_name,
                        end_time: raw.end_time,
                        desc: raw.desc,
                    };
                    results.push(blacklist);
                }
                Err(e) => {
                    warn!(blacklist_id = %id, error = %e, "Parse Redis blacklist failed, skip record");
                }
            }
        }

        Ok(results)
    }
}
