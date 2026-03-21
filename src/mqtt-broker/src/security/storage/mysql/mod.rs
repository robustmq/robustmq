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
use crate::security::AuthStorageAdapter;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::get_blacklist_type_by_str;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::{enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction, tools::now_second};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::storage::MysqlConfig;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::tenant::DEFAULT_TENANT;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::Row;
use third_driver::mysql::{build_mysql_conn_pool, MysqlPool};

mod schema;

pub struct MySQLAuthStorageAdapter {
    pool: MysqlPool,
    config: MysqlConfig,
}

impl MySQLAuthStorageAdapter {
    pub fn new(config: MysqlConfig) -> Result<Self, MqttBrokerError> {
        let addr = format!(
            "mysql://{}:{}@{}/{}",
            config.username, config.password, config.mysql_addr, config.database
        );
        let pool = build_mysql_conn_pool(&addr)?;
        Ok(MySQLAuthStorageAdapter { pool, config })
    }

    fn table_user(&self) -> &'static str {
        "mqtt_user"
    }

    fn table_acl(&self) -> &'static str {
        "mqtt_acl"
    }

    fn table_blacklist(&self) -> &'static str {
        "mqtt_blacklist"
    }

    fn parse_created_to_seconds(created: Option<String>) -> u64 {
        if let Some(value) = created {
            // MySQL DATETIME format used by current schema: "YYYY-MM-DD HH:MM:SS".
            if let Ok(dt) = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S") {
                return dt.and_utc().timestamp().max(0) as u64;
            }
        }
        now_second()
    }

    fn user_query(&self) -> String {
        if self.config.query_user.trim().is_empty() {
            format!(
                "select username as username, password as password, salt as salt, is_superuser as is_superuser, created as created from {}",
                self.table_user()
            )
        } else {
            self.config.query_user.clone()
        }
    }

    fn acl_query(&self) -> String {
        if self.config.query_acl.trim().is_empty() {
            format!(
                "select permission as permission, ipaddr as ipaddr, username as username, clientid as clientid, access as access, topic as topic from {}",
                self.table_acl()
            )
        } else {
            self.config.query_acl.clone()
        }
    }

    fn blacklist_query(&self) -> String {
        if self.config.query_blacklist.trim().is_empty() {
            format!(
                "select blacklist_type as blacklist_type, resource_name as resource_name, end_time as end_time, `desc` as `desc` from {}",
                self.table_blacklist()
            )
        } else {
            self.config.query_blacklist.clone()
        }
    }
}

#[async_trait]
impl AuthStorageAdapter for MySQLAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = self.user_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let results = DashMap::with_capacity(rows.len());
        for mut row in rows {
            let username: String = row.take("username").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: username".to_string())
            })?;
            let password: String = row.take("password").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: password".to_string())
            })?;
            let salt: Option<String> = row
                .take("salt")
                .ok_or_else(|| MqttBrokerError::CommonError("missing column: salt".to_string()))?;
            let is_superuser: u8 = row.take("is_superuser").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: is_superuser".to_string())
            })?;
            let created: Option<String> = row.take("created").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: created".to_string())
            })?;

            let user = MqttUser {
                tenant: DEFAULT_TENANT.to_string(),
                username: username.clone(),
                password,
                salt,
                is_superuser: is_superuser == 1,
                create_time: Self::parse_created_to_seconds(created),
            };
            results.insert(username, user);
        }
        return Ok(results);
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = self.acl_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let mut results = Vec::with_capacity(rows.len());
        for mut row in rows {
            let permission: u8 = row.take("permission").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: permission".to_string())
            })?;
            let ipaddr: String = row.take("ipaddr").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: ipaddr".to_string())
            })?;
            let username: String = row.take("username").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: username".to_string())
            })?;
            let clientid: String = row.take("clientid").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: clientid".to_string())
            })?;
            let access: u8 = row.take("access").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: access".to_string())
            })?;
            let topic: Option<String> = row
                .take("topic")
                .ok_or_else(|| MqttBrokerError::CommonError("missing column: topic".to_string()))?;

            let name: String = row.take("name").unwrap_or_default();
            let desc: String = row.take("desc").unwrap_or_default();

            let acl = MqttAcl {
                name,
                desc,
                tenant: DEFAULT_TENANT.to_string(),
                permission: match permission {
                    0 => MqttAclPermission::Deny,
                    1 => MqttAclPermission::Allow,
                    _ => return Err(MqttBrokerError::InvalidAclPermission),
                },
                resource_type: match username.is_empty() {
                    true => MqttAclResourceType::ClientId,
                    false => MqttAclResourceType::User,
                },
                resource_name: match username.is_empty() {
                    true => clientid,
                    false => username,
                },
                topic: topic.unwrap_or_default(),
                ip: ipaddr,
                action: match access {
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
        return Ok(results);
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = self.blacklist_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let mut results = Vec::with_capacity(rows.len());
        for mut row in rows {
            let blacklist_type: String = row.take("blacklist_type").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: blacklist_type".to_string())
            })?;
            let resource_name: String = row.take("resource_name").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: resource_name".to_string())
            })?;
            let end_time: u64 = row.take("end_time").ok_or_else(|| {
                MqttBrokerError::CommonError("missing column: end_time".to_string())
            })?;
            let desc: Option<String> = row
                .take("desc")
                .ok_or_else(|| MqttBrokerError::CommonError("missing column: desc".to_string()))?;

            let blacklist = MqttAclBlackList {
                tenant: DEFAULT_TENANT.to_string(),
                blacklist_type: get_blacklist_type_by_str(&blacklist_type)?,
                resource_name,
                end_time,
                desc: desc.unwrap_or_default(),
            };
            results.push(blacklist);
        }
        return Ok(results);
    }
}

#[cfg(test)]
mod tests {
    use metadata_struct::mqtt::auth::storage::MysqlConfig;
    use r2d2_mysql::mysql::params;
    use r2d2_mysql::mysql::prelude::Queryable;
    use third_driver::mysql::build_mysql_conn_pool;

    use super::schema::TAuthUser;
    use super::MySQLAuthStorageAdapter;
    use crate::security::AuthStorageAdapter;

    #[tokio::test]
    #[ignore]
    async fn read_all_user_test() {
        let config = MysqlConfig {
            mysql_addr: "127.0.0.1:3306".to_string(),
            database: "mqtt".to_string(),
            username: "root".to_string(),
            password: "123456".to_string(),
            ..Default::default()
        };

        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt".to_string();
        init_user(&addr);
        let auth_mysql = MySQLAuthStorageAdapter::new(config).unwrap();
        let result = auth_mysql.read_all_user().await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.contains_key("robustmq"));
        let user = res.get("robustmq").unwrap();
        assert_eq!(user.password, "robustmq@2024");
    }

    fn init_user(addr: &str) {
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mut conn = pool.get().unwrap();
        let values = [TAuthUser {
            username: username(),
            password: password(),
            ..Default::default()
        }];
        conn.exec_batch(
            format!("REPLACE INTO {}(username,password,salt,is_superuser,created) VALUES (:username,:password,:salt,:is_superuser,:created)",
            "mqtt_user"),
            values.iter().map(|p| {
                params! {
                    "username" => p.username.clone(),
                    "password" => p.password.clone(),
                    "salt" => "".to_string(),
                    "is_superuser" => 1,
                    "created" => "2024-10-01 10:10:10",
                }
            }),
        ).unwrap();
    }

    fn username() -> String {
        "robustmq".to_string()
    }

    fn password() -> String {
        "robustmq@2024".to_string()
    }
}
