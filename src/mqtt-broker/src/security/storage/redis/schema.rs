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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RedisAuthUser {
    pub username: String,
    pub password: String,
    pub salt: String,
    pub is_superuser: u8,
    pub created: Option<u64>,
}

impl RedisAuthUser {
    pub fn redis_user_key(username: &str) -> String {
        format!("mqtt:user:{}", username)
    }

    pub fn redis_users_key() -> String {
        "mqtt:users".to_string()
    }

    pub fn from_redis_hash(
        username: String,
        fields: HashMap<String, String>,
    ) -> Result<Self, String> {
        let password = fields
            .get("password")
            .ok_or("Missing password field")?
            .clone();

        let salt = fields.get("salt").unwrap_or(&String::new()).clone();

        let is_superuser = fields
            .get("is_superuser")
            .ok_or("Missing is_superuser field")?
            .parse::<u8>()
            .map_err(|_| "Invalid is_superuser value")?;

        Ok(RedisAuthUser {
            username,
            password,
            salt,
            is_superuser,
            created: fields.get("created").and_then(|v| v.parse::<u64>().ok()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RedisAuthAcl {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub username: String,
    pub permission: u8,
    pub ipaddr: String,
    pub clientid: String,
    pub access: u8,
    pub topic: String,
}

impl RedisAuthAcl {
    pub fn redis_acl_key(id: &str) -> String {
        format!("mqtt:acl:{}", id)
    }

    pub fn redis_acls_key() -> String {
        "mqtt:acls".to_string()
    }

    pub fn from_redis_hash(id: String, fields: HashMap<String, String>) -> Result<Self, String> {
        let name = fields.get("name").cloned().unwrap_or_else(|| id.clone());

        let desc = fields.get("desc").cloned().unwrap_or_default();

        let username = fields.get("username").unwrap_or(&String::new()).clone();

        let permission = fields
            .get("permission")
            .ok_or("Missing permission field")?
            .parse::<u8>()
            .map_err(|_| "Invalid permission value")?;

        let ipaddr = fields.get("ipaddr").unwrap_or(&String::new()).clone();

        let clientid = fields.get("clientid").unwrap_or(&String::new()).clone();

        let access = fields
            .get("access")
            .ok_or("Missing access field")?
            .parse::<u8>()
            .map_err(|_| "Invalid access value")?;

        let topic = fields.get("topic").unwrap_or(&String::new()).clone();

        Ok(RedisAuthAcl {
            id,
            name,
            desc,
            username,
            permission,
            ipaddr,
            clientid,
            access,
            topic,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RedisAuthBlacklist {
    pub id: String,
    pub name: String,
    pub blacklist_type: String,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

impl RedisAuthBlacklist {
    pub fn redis_blacklist_key(id: &str) -> String {
        format!("mqtt:blacklist:{}", id)
    }

    pub fn redis_blacklists_key() -> String {
        "mqtt:blacklists".to_string()
    }

    pub fn from_redis_hash(id: String, fields: HashMap<String, String>) -> Result<Self, String> {
        let blacklist_type = fields
            .get("blacklist_type")
            .ok_or("Missing blacklist_type field")?
            .clone();

        let resource_name = fields
            .get("resource_name")
            .ok_or("Missing resource_name field")?
            .clone();

        let end_time = fields
            .get("end_time")
            .ok_or("Missing end_time field")?
            .parse::<u64>()
            .map_err(|_| "Invalid end_time value")?;

        let desc = fields.get("desc").cloned().unwrap_or_default();
        let name = fields.get("name").cloned().unwrap_or_else(|| id.clone());

        Ok(RedisAuthBlacklist {
            id,
            name,
            blacklist_type,
            resource_name,
            end_time,
            desc,
        })
    }
}
