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
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::{
    MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
};
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use mysql::prelude::Queryable;
use mysql::Pool;
use third_driver::mysql::build_mysql_conn_pool;

use super::AuthStorageAdapter;

mod schema;
pub struct MySQLAuthStorageAdapter {
    pool: Pool,
}

impl MySQLAuthStorageAdapter {
    pub fn new(addr: String) -> Self {
        let pool = match build_mysql_conn_pool(&addr) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        MySQLAuthStorageAdapter { pool }
    }

    fn table_user(&self) -> String {
        "mqtt_user".to_string()
    }

    fn table_acl(&self) -> String {
        "mqtt_acl".to_string()
    }
}

#[async_trait]
impl AuthStorageAdapter for MySQLAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select username,password,salt,is_superuser,created from {}",
                    self.table_user()
                );
                let data: Vec<(String, String, Option<String>, u8, Option<String>)> =
                    conn.query(sql).unwrap();
                let results = DashMap::with_capacity(2);
                for raw in data {
                    let user = MqttUser {
                        username: raw.0.clone(),
                        password: raw.1.clone(),
                        is_superuser: raw.3 == 1,
                    };
                    results.insert(raw.0.clone(), user);
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select username,password,salt,is_superuser,created from {} where username='{}'",
                    self.table_user(),
                    username
                );
                let data: Vec<(String, String, Option<String>, u8, Option<String>)> =
                    conn.query(sql).unwrap();
                if let Some(value) = data.first() {
                    return Ok(Some(MqttUser {
                        username: value.0.clone(),
                        password: value.1.clone(),
                        is_superuser: value.3 == 1,
                    }));
                }
                return Ok(None);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn save_user(&self, user_info: MqttUser) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "insert into {} ( `username`, `password`, `is_superuser`, `salt`) values ('{}', '{}', '{}', null);",
                    self.table_user(),
                    user_info.username,
                    user_info.password,
                    user_info.is_superuser as i32,
                );
                let _data: Vec<(String, String, Option<String>, u8)> = conn.query(sql).unwrap();
                return Ok(());
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn delete_user(&self, username: String) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "delete from {} where username = '{}';",
                    self.table_user(),
                    username
                );
                let _data: Vec<(String, String, Option<String>, u8, Option<String>)> =
                    conn.query(sql).unwrap();
                return Ok(());
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select allow, ipaddr, username, clientid, access, topic from {}",
                    self.table_acl()
                );
                let data: Vec<(u8, String, String, String, u8, Option<String>)> =
                    conn.query(sql).unwrap();
                let mut results = Vec::new();
                for raw in data {
                    let acl = MqttAcl {
                        permission: match raw.0 {
                            0 => MqttAclPermission::Deny,
                            1 => MqttAclPermission::Allow,
                            _ => {
                                return Err(CommonError::CommonError(
                                    "invalid acl permission".to_string(),
                                ))
                            }
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
                            _ => {
                                return Err(CommonError::CommonError(
                                    "invalid acl action".to_string(),
                                ))
                            }
                        },
                    };
                    results.push(acl);
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn save_acl(&self, acl: MqttAcl) -> Result<(), CommonError> {
        let allow: u8 = match acl.permission {
            MqttAclPermission::Allow => 1,
            MqttAclPermission::Deny => 0,
        };
        let (username, clientid) = match acl.resource_type.clone() {
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
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "insert into {} (allow, ipaddr, username, clientid, access, topic) values ('{}', '{}', '{}', '{}', '{}', '{}');",
                    self.table_acl(),
                    allow,
                    acl.ip,
                    username,
                    clientid,
                    access,
                    acl.topic,
                );
                let _data: Vec<(
                    u8,
                    String,
                    Option<String>,
                    Option<String>,
                    u8,
                    Option<String>,
                )> = conn.query(sql).unwrap();
                return Ok(());
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn delete_acl(&self, acl: MqttAcl) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = match acl.resource_type.clone() {
                    MqttAclResourceType::ClientId => format!(
                        "delete from {} where clientid = '{}';",
                        self.table_acl(),
                        acl.resource_name
                    ),
                    MqttAclResourceType::User => format!(
                        "delete from {} where username = '{}';",
                        self.table_acl(),
                        acl.resource_name
                    ),
                };
                let _data: Vec<(
                    u8,
                    String,
                    Option<String>,
                    Option<String>,
                    u8,
                    Option<String>,
                )> = conn.query(sql).unwrap();
                return Ok(());
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, CommonError> {
        return Ok(Vec::new());
    }
}

#[cfg(test)]
mod tests {

    use mysql::params;
    use mysql::prelude::Queryable;
    use third_driver::mysql::build_mysql_conn_pool;

    use super::schema::TAuthUser;
    use super::MySQLAuthStorageAdapter;
    use crate::security::AuthStorageAdapter;

    #[tokio::test]
    #[ignore]
    async fn read_all_user_test() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt".to_string();
        init_user(&addr);
        let auth_mysql = MySQLAuthStorageAdapter::new(addr);
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
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt".to_string();
        init_user(&addr);
        let auth_mysql = MySQLAuthStorageAdapter::new(addr);
        let username = "robustmq".to_string();
        let result = auth_mysql.get_user(username).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        let user = res.unwrap();
        assert_eq!(user.password, "robustmq@2024");
    }

    fn init_user(addr: &str) {
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mut conn = pool.get_conn().unwrap();
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
