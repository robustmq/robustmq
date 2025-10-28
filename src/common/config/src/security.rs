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

/// TODO: add validator

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthnConfig {
    pub authn_type: String, // Password-Based/JWT/SCRAM/GSSAPI/ClientInfo...
    pub jwt_config: Option<JwtConfig>,
    pub password_based_config: Option<PasswordBasedConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AuthzConfig {
    pub storage_config: StorageConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JwtConfig {
    pub jwt_source: String,                  // password/username
    pub jwt_encryption: String,              // hmac-based/public-key
    pub secret: Option<String>,              // hmac-based need
    pub secret_base64_encoded: Option<bool>, // hmac-based need
    pub public_key: Option<String>,          // public-key need
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PasswordBasedConfig {
    pub storage_config: StorageConfig,
    pub password_config: PasswordConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordConfig {
    pub credential_type: String, // username/clientid (only for placement)
    pub algorithm: String,       // plain/md5/sha/sha256/sha512/bcrypt/pbkdf2
    pub salt_position: Option<String>, // disable/prefix/suffix
    pub salt_rounds: Option<u32>, // bcrypt need
    pub mac_fun: Option<String>, // pbkdf2 need
    pub iterations: Option<u32>, // pbkdf2 need
    pub dk_length: Option<u32>,  // pbkdf2 need
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub storage_type: String, // file(only for authz)/placement/mysql/postgresql/redis/http
    pub placement_config: Option<PlacementConfig>,
    pub mysql_config: Option<MysqlConfig>,
    pub postgres_config: Option<PostgresConfig>,
    pub redis_config: Option<RedisConfig>,
    pub http_config: Option<HttpConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlacementConfig {
    pub journal_addr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlConfig {
    pub mysql_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresConfig {
    pub postgre_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConfig {
    pub redis_addr: String,
    pub mode: String, // Single/Cluster/Sentinel
    pub database: i32,
    pub password: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HttpConfig {
    pub url: String,
    pub method: String, // GET/POST
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
}

impl Default for AuthnConfig {
    fn default() -> Self {
        Self {
            authn_type: "password_based".to_string(),
            jwt_config: None,
            password_based_config: Some(PasswordBasedConfig::default()),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            jwt_source: "password".to_string(),
            jwt_encryption: "hmac-based".to_string(),
            secret: Some("mqtt_secret".to_string()),
            secret_base64_encoded: Some(false),
            public_key: None,
        }
    }
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            credential_type: "username".to_string(),
            algorithm: "plain".to_string(),
            salt_position: Some("disable".to_string()),
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: "placement".to_string(),
            placement_config: Some(PlacementConfig::default()),
            mysql_config: None,
            postgres_config: None,
            redis_config: None,
            http_config: None,
        }
    }
}

impl Default for PlacementConfig {
    fn default() -> Self {
        Self {
            journal_addr: "".to_string(),
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
            query: "SELECT password_hash, salt FROM mqtt_user where username = ${username} LIMIT 1"
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
            query: "SELECT password_hash, salt FROM mqtt_user where username = ${username} LIMIT 1"
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
            query: "HMGET mqtt_user:${username} password_hash salt".to_string(),
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
            headers: Some(headers),
            body: Some(body),
        }
    }
}
