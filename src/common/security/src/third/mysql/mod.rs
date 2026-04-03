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

#![allow(clippy::result_large_err)]
use crate::third::storage_trait::AuthStorageAdapter;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::acl::{EnumAclAction, EnumAclPermission, EnumAclResourceType};
use metadata_struct::auth::blacklist::get_blacklist_type_by_str;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::mqtt::auth::storage::MysqlConfig;
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
    pub fn new(config: MysqlConfig) -> Result<Self, CommonError> {
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
    async fn read_all_user(&self) -> Result<DashMap<String, SecurityUser>, CommonError> {
        let mut conn = self.pool.get()?;
        let sql = self.user_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let results = DashMap::with_capacity(rows.len());
        for mut row in rows {
            let username: String = row
                .take("username")
                .ok_or_else(|| CommonError::CommonError("missing column: username".to_string()))?;
            let password: String = row
                .take("password")
                .ok_or_else(|| CommonError::CommonError("missing column: password".to_string()))?;
            let salt: Option<String> = row
                .take("salt")
                .ok_or_else(|| CommonError::CommonError("missing column: salt".to_string()))?;
            let is_superuser: u8 = row.take("is_superuser").ok_or_else(|| {
                CommonError::CommonError("missing column: is_superuser".to_string())
            })?;
            let created: Option<String> = row
                .take("created")
                .ok_or_else(|| CommonError::CommonError("missing column: created".to_string()))?;

            let user = SecurityUser {
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

    async fn read_all_acl(&self) -> Result<Vec<SecurityAcl>, CommonError> {
        let mut conn = self.pool.get()?;
        let sql = self.acl_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let mut results = Vec::with_capacity(rows.len());
        for mut row in rows {
            let permission: u8 = row.take("permission").ok_or_else(|| {
                CommonError::CommonError("missing column: permission".to_string())
            })?;
            let ipaddr: String = row
                .take("ipaddr")
                .ok_or_else(|| CommonError::CommonError("missing column: ipaddr".to_string()))?;
            let username: String = row
                .take("username")
                .ok_or_else(|| CommonError::CommonError("missing column: username".to_string()))?;
            let clientid: String = row
                .take("clientid")
                .ok_or_else(|| CommonError::CommonError("missing column: clientid".to_string()))?;
            let access: u8 = row
                .take("access")
                .ok_or_else(|| CommonError::CommonError("missing column: access".to_string()))?;
            let topic: Option<String> = row
                .take("topic")
                .ok_or_else(|| CommonError::CommonError("missing column: topic".to_string()))?;

            let name: String = row.take("name").unwrap_or_default();
            let desc: String = row.take("desc").unwrap_or_default();

            let acl = SecurityAcl {
                name,
                desc,
                tenant: DEFAULT_TENANT.to_string(),
                permission: match permission {
                    0 => EnumAclPermission::Deny,
                    1 => EnumAclPermission::Allow,
                    _ => return Err(CommonError::InvalidAclPermission),
                },
                resource_type: match username.is_empty() {
                    true => EnumAclResourceType::ClientId,
                    false => EnumAclResourceType::User,
                },
                resource_name: match username.is_empty() {
                    true => clientid,
                    false => username,
                },
                topic: topic.unwrap_or_default(),
                ip: ipaddr,
                action: match access {
                    0 => EnumAclAction::All,
                    1 => EnumAclAction::Subscribe,
                    2 => EnumAclAction::Publish,
                    3 => EnumAclAction::PubSub,
                    4 => EnumAclAction::Retain,
                    5 => EnumAclAction::Qos,
                    _ => return Err(CommonError::InvalidAclAction),
                },
            };
            results.push(acl);
        }
        return Ok(results);
    }

    async fn read_all_blacklist(&self) -> Result<Vec<SecurityBlackList>, CommonError> {
        let mut conn = self.pool.get()?;
        let sql = self.blacklist_query();
        let rows: Vec<Row> = conn.query(sql)?;
        let mut results = Vec::with_capacity(rows.len());
        for mut row in rows {
            let blacklist_type: String = row.take("blacklist_type").ok_or_else(|| {
                CommonError::CommonError("missing column: blacklist_type".to_string())
            })?;
            let resource_name: String = row.take("resource_name").ok_or_else(|| {
                CommonError::CommonError("missing column: resource_name".to_string())
            })?;
            let end_time: u64 = row
                .take("end_time")
                .ok_or_else(|| CommonError::CommonError("missing column: end_time".to_string()))?;
            let desc: Option<String> = row
                .take("desc")
                .ok_or_else(|| CommonError::CommonError("missing column: desc".to_string()))?;

            let name: Option<String> = row.take("name");
            let blacklist = SecurityBlackList {
                name: name.unwrap_or_default(),
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
    use super::schema::TAuthUser;
    use super::MySQLAuthStorageAdapter;
    use crate::third::storage_trait::AuthStorageAdapter;
    use metadata_struct::mqtt::auth::storage::MysqlConfig;
    use r2d2_mysql::mysql::params;
    use r2d2_mysql::mysql::prelude::Queryable;
    use third_driver::mysql::build_mysql_conn_pool;

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
