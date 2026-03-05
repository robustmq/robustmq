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
use async_trait::async_trait;
use chrono::NaiveDateTime;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::{enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction, tools::now_second};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::storage::MysqlConfig;
use metadata_struct::mqtt::user::MqttUser;
use r2d2_mysql::mysql::params;
use r2d2_mysql::mysql::prelude::Queryable;
use third_driver::mysql::{build_mysql_conn_pool, MysqlPool};

mod schema;
pub struct MySQLAuthStorageAdapter {
    pool: MysqlPool,
}

impl MySQLAuthStorageAdapter {
    pub fn new(config: MysqlConfig) -> Result<Self, MqttBrokerError> {
        let addr = format!(
            "mysql://{}:{}@{}/{}",
            config.username, config.password, config.mysql_addr, config.database
        );
        let pool = build_mysql_conn_pool(&addr)?;
        Ok(MySQLAuthStorageAdapter { pool })
    }

    fn table_user(&self) -> &'static str {
        "mqtt_user"
    }

    fn table_acl(&self) -> &'static str {
        "mqtt_acl"
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
}

#[async_trait]
impl AuthStorageAdapter for MySQLAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select username,password,salt,is_superuser,created from {}",
            self.table_user()
        );
        let data: Vec<(String, String, Option<String>, u8, Option<String>)> = conn.query(sql)?;
        let results = DashMap::with_capacity(data.len());
        for raw in data {
            let user = MqttUser {
                username: raw.0.clone(),
                password: raw.1.clone(),
                salt: raw.2.clone(),
                is_superuser: raw.3 == 1,
                create_time: Self::parse_created_to_seconds(raw.4.clone()),
            };
            results.insert(raw.0.clone(), user);
        }
        return Ok(results);
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select permission, ipaddr, username, clientid, access, topic from {}",
            self.table_acl()
        );
        let data: Vec<(u8, String, String, String, u8, Option<String>)> = conn.query(sql)?;
        let mut results = Vec::new();
        for raw in data {
            let acl = MqttAcl {
                permission: match raw.0 {
                    0 => MqttAclPermission::Deny,
                    1 => MqttAclPermission::Allow,
                    _ => return Err(MqttBrokerError::InvalidAclPermission),
                },
                resource_type: match raw.2.clone().is_empty() {
                    true => MqttAclResourceType::ClientId,
                    false => MqttAclResourceType::User,
                },
                resource_name: match raw.2.clone().is_empty() {
                    true => raw.3.clone(),
                    false => raw.2.clone(),
                },
                topic: raw.5.clone().unwrap_or(String::new()),
                ip: raw.1.clone(),
                action: match raw.4 {
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
        return Ok(Vec::new());
    }

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select username,password,salt,is_superuser,created from {} where username=:username",
            self.table_user()
        );
        let data: Option<(String, String, Option<String>, u8, Option<String>)> =
            conn.exec_first(sql, params! { "username" => username })?;
        if let Some(value) = data {
            return Ok(Some(MqttUser {
                username: value.0,
                password: value.1,
                salt: value.2,
                is_superuser: value.3 == 1,
                create_time: Self::parse_created_to_seconds(value.4),
            }));
        }
        return Ok(None);
    }

    async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "insert into {} (`username`, `password`, `is_superuser`, `salt`) values (:username, :password, :is_superuser, :salt)",
            self.table_user(),
        );
        conn.exec_drop(
            sql,
            params! {
                "username" => user_info.username,
                "password" => user_info.password,
                "is_superuser" => user_info.is_superuser as i32,
                "salt" => user_info.salt,
            },
        )?;
        return Ok(());
    }

    async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "delete from {} where username = :username",
            self.table_user(),
        );
        conn.exec_drop(sql, params! { "username" => username })?;
        return Ok(());
    }

    async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
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

        let mut conn = self.pool.get()?;
        let sql = format!(
            "insert into {} (permission, ipaddr, username, clientid, access, topic) values (:permission, :ipaddr, :username, :clientid, :access, :topic)",
            self.table_acl(),
        );
        conn.exec_drop(
            sql,
            params! {
                "permission" => permission,
                "ipaddr" => acl.ip,
                "username" => username,
                "clientid" => clientid,
                "access" => access,
                "topic" => acl.topic,
            },
        )?;

        return Ok(());
    }

    async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let (sql, params) = match acl.resource_type {
            MqttAclResourceType::ClientId => (
                format!(
                    "delete from {} where clientid = :resource_name",
                    self.table_acl()
                ),
                params! { "resource_name" => acl.resource_name },
            ),
            MqttAclResourceType::User => (
                format!(
                    "delete from {} where username = :resource_name",
                    self.table_acl()
                ),
                params! { "resource_name" => acl.resource_name },
            ),
        };
        conn.exec_drop(sql, params)?;
        return Ok(());
    }
    async fn save_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        return Ok(());
    }
    async fn delete_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        return Ok(());
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

    #[tokio::test]
    #[ignore]
    async fn get_user_test() {
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
        let username = "robustmq".to_string();
        let result = auth_mysql.get_user(username).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        let user = res.unwrap();
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
