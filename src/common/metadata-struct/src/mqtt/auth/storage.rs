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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub storage_type: String, // file(only for authz)/placement/mysql/postgresql/redis/http
    pub mysql_config: Option<MysqlConfig>,
    pub postgres_config: Option<PostgresConfig>,
    pub redis_config: Option<RedisConfig>,
    pub mongodb_config: Option<MongoDBConfig>,
    pub http_config: Option<HttpConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlConfig {
    pub mysql_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresConfig {
    pub postgre_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConfig {
    pub redis_addr: String,
    pub mode: String, // Single/Cluster/Sentinel
    pub database: i32,
    pub password: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MongoDBConfig {
    pub mongodb_uri: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub collection_user: String,
    pub collection_acl: String,
    pub collection_blacklist: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HttpConfig {
    pub url: String,
    pub method: String, // GET/POST
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: "placement".to_string(),
            mysql_config: None,
            postgres_config: None,
            redis_config: None,
            mongodb_config: None,
            http_config: None,
        }
    }
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            mysql_addr: "127.0.0.1:3306".to_string(),
            database: "mqtt_user".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            query_user: "SELECT username AS username, password AS password, salt AS salt, is_superuser AS is_superuser, created AS created FROM mqtt_user".to_string(),
            query_acl: "SELECT permission AS permission, ipaddr AS ipaddr, username AS username, clientid AS clientid, access AS access, topic AS topic FROM mqtt_acl".to_string(),
            query_blacklist:
                "SELECT blacklist_type AS blacklist_type, resource_name AS resource_name, end_time AS end_time, `desc` AS `desc` FROM mqtt_blacklist"
                    .to_string(),
        }
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            postgre_addr: "127.0.0.1:5432".to_string(),
            database: "mqtt_user".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            query_user: "SELECT username AS username, password AS password, salt AS salt, is_superuser AS is_superuser, created::text AS created FROM mqtt_user".to_string(),
            query_acl: "SELECT permission AS permission, ipaddr AS ipaddr, username AS username, clientid AS clientid, access AS access, topic AS topic FROM mqtt_acl".to_string(),
            query_blacklist:
                "SELECT blacklist_type AS blacklist_type, resource_name AS resource_name, end_time AS end_time, \"desc\" AS \"desc\" FROM mqtt_blacklist"
                    .to_string(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            redis_addr: "127.0.0.1:6379".to_string(),
            mode: "Single".to_string(),
            database: 0,
            password: "".to_string(),
            query_user: "SMEMBERS mqtt:users".to_string(),
            query_acl: "SMEMBERS mqtt:acls".to_string(),
            query_blacklist: "SMEMBERS mqtt:blacklists".to_string(),
        }
    }
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            mongodb_uri: "mongodb://127.0.0.1:27017".to_string(),
            database: "mqtt".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            collection_user: "mqtt_user".to_string(),
            collection_acl: "mqtt_acl".to_string(),
            collection_blacklist: "mqtt_blacklist".to_string(),
            query_user: "{}".to_string(),
            query_acl: "{}".to_string(),
            query_blacklist: "{}".to_string(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let mut body = HashMap::new();
        body.insert("username".to_string(), "${username}".to_string());
        body.insert("password".to_string(), "${password}".to_string());

        Self {
            url: "http://127.0.0.1:8080/auth".to_string(),
            method: "POST".to_string(),
            query_user: "".to_string(),
            query_acl: "".to_string(),
            query_blacklist: "".to_string(),
            headers: Some(headers),
            body: Some(body),
        }
    }
}
