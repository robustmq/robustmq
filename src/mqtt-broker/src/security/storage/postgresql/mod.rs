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

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;
use crate::security::AuthStorageAdapter;
use axum::async_trait;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::{enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction, tools::now_second};
use common_config::security::PostgresConfig;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use third_driver::postgresql::{build_postgresql_conn_pool, PostgresPool};

mod schema;

pub struct PostgresqlAuthStorageAdapter {
    pool: PostgresPool,
    #[allow(dead_code)]
    config: PostgresConfig,
}

impl PostgresqlAuthStorageAdapter {
    pub fn new(config: PostgresConfig) -> Self {
        let addr = format!(
            "postgres://{}:{}@{}/{}",
            config.username, config.password, config.postgre_addr, config.database
        );

        let pool = match build_postgresql_conn_pool(&addr) {
            Ok(data) => data,
            Err(e) => {
                panic!("Failed to create PostgreSQL connection pool: {}", e);
            }
        };
        PostgresqlAuthStorageAdapter { pool, config }
    }

    fn table_user(&self) -> String {
        "mqtt_user".to_string()
    }

    fn table_acl(&self) -> String {
        "mqtt_acl".to_string()
    }
}

#[async_trait]
impl AuthStorageAdapter for PostgresqlAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select username,password,salt,is_superuser,created from {}",
            self.table_user()
        );
        let rows = conn.query(&sql, &[])?;
        let results = DashMap::with_capacity(2);
        for row in rows {
            let username: String = row.get(0);
            let password: String = row.get(1);
            let salt: Option<String> = row.get(2);
            let is_superuser: i32 = row.get(3);
            let user = MqttUser {
                username: username.clone(),
                password,
                salt,
                is_superuser: is_superuser == 1,
                // todo bugfix
                create_time: now_second(),
            };
            results.insert(username, user);
        }
        return Ok(results);
    }

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select permission, ipaddr, username, clientid, access, topic from {}",
            self.table_acl()
        );
        let rows = conn.query(&sql, &[])?;
        let mut results = Vec::new();
        for row in rows {
            let permission: i32 = row.get(0);
            let ipaddr: String = row.get(1);
            let username: String = row.get(2);
            let clientid: String = row.get(3);
            let access: i32 = row.get(4);
            let topic: Option<String> = row.get(5);

            let acl = MqttAcl {
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
        return Ok(Vec::new());
    }

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "select username,password,salt,is_superuser,created from {} where username=$1",
            self.table_user()
        );
        let rows = conn.query(&sql, &[&username])?;
        if let Some(row) = rows.first() {
            let username: String = row.get(0);
            let password: String = row.get(1);
            let salt: Option<String> = row.get(2);
            let is_superuser: i32 = row.get(3);
            return Ok(Some(MqttUser {
                username,
                password,
                salt,
                is_superuser: is_superuser == 1,
                // todo bugfix
                create_time: now_second(),
            }));
        }
        return Ok(None);
    }

    async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let sql = format!(
            "insert into {} (username, password, is_superuser, salt) values ($1, $2, $3, $4)",
            self.table_user()
        );
        conn.execute(
            &sql,
            &[
                &user_info.username,
                &user_info.password,
                &(user_info.is_superuser as i32),
                &user_info.salt,
            ],
        )?;
        return Ok(());
    }

    async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let sql = format!("delete from {} where username = $1", self.table_user());
        conn.execute(&sql, &[&username])?;
        return Ok(());
    }

    async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let permission: i32 = match acl.permission {
            MqttAclPermission::Allow => 1,
            MqttAclPermission::Deny => 0,
        };
        let (username, clientid) = match acl.resource_type {
            MqttAclResourceType::ClientId => (String::new(), acl.resource_name),
            MqttAclResourceType::User => (acl.resource_name, String::new()),
        };
        let access: i32 = match acl.action {
            MqttAclAction::All => 0,
            MqttAclAction::Subscribe => 1,
            MqttAclAction::Publish => 2,
            MqttAclAction::PubSub => 3,
            MqttAclAction::Retain => 4,
            MqttAclAction::Qos => 5,
        };

        let mut conn = self.pool.get()?;
        let sql = format!(
            "insert into {} (permission, ipaddr, username, clientid, access, topic) values ($1, $2, $3, $4, $5, $6)",
            self.table_acl()
        );

        conn.execute(
            &sql,
            &[
                &permission,
                &acl.ip,
                &username,
                &clientid,
                &access,
                &acl.topic,
            ],
        )?;

        return Ok(());
    }

    async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        let mut conn = self.pool.get()?;
        let sql = match acl.resource_type {
            MqttAclResourceType::ClientId => {
                format!("delete from {} where clientid = $1", self.table_acl())
            }
            MqttAclResourceType::User => {
                format!("delete from {} where username = $1", self.table_acl())
            }
        };
        conn.execute(&sql, &[&acl.resource_name])?;
        return Ok(());
    }
    async fn save_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        return Ok(());
    }
    async fn delete_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        return Ok(());
    }
}
