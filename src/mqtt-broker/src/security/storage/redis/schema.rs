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
}

impl RedisAuthUser {
    pub fn redis_user_key(username: &str) -> String {
        format!("mqtt:user:{}", username)
    }

    pub fn redis_users_key() -> String {
        "mqtt:users".to_string()
    }

    pub fn to_redis_hash(&self) -> HashMap<String, String> {
        let mut hash = HashMap::new();
        hash.insert("password".to_string(), self.password.clone());
        hash.insert("salt".to_string(), self.salt.clone());
        hash.insert("is_superuser".to_string(), self.is_superuser.to_string());
        hash
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
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RedisAuthAcl {
    pub id: String,
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

    pub fn to_redis_hash(&self) -> HashMap<String, String> {
        let mut hash = HashMap::new();
        hash.insert("username".to_string(), self.username.clone());
        hash.insert("permission".to_string(), self.permission.to_string());
        hash.insert("ipaddr".to_string(), self.ipaddr.clone());
        hash.insert("clientid".to_string(), self.clientid.clone());
        hash.insert("access".to_string(), self.access.to_string());
        hash.insert("topic".to_string(), self.topic.clone());
        hash
    }

    pub fn from_redis_hash(id: String, fields: HashMap<String, String>) -> Result<Self, String> {
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
            username,
            permission,
            ipaddr,
            clientid,
            access,
            topic,
        })
    }

    pub fn generate_id(username: &str, clientid: &str, topic: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}:{}", username, clientid, topic).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
