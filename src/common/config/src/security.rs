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
    pub jwt_config: Option<JwtConfig>,
    pub password_config: Option<PasswordConfig>,
    pub http_config: Option<HttpConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AuthzConfig {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JwtConfig {
    pub jwt_source: String,                  // password/username
    pub jwt_encryption: String,              // hmac-based/public-key
    pub secret: Option<String>,              // hmac-based need
    pub secret_base64_encoded: Option<bool>, // hmac-based need
    pub public_key: Option<String>,          // public-key need
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordConfig {
    pub credential_type: String,       // password/username
    pub algorithm: String,             // plain/md5/sha/sha256/sha512/bcrypt/pbkdf2
    pub salt_position: Option<String>, // disable/prefix/suffix
    pub salt_rounds: Option<u32>,      // bcrypt need
    pub mac_fun: Option<String>,       // pbkdf2 need
    pub iterations: Option<u32>,       // pbkdf2 need
    pub dk_length: Option<u32>,        // pbkdf2 need
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
            jwt_config: None,
            password_config: Some(PasswordConfig::default()),
            http_config: None,
        }
    }
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            credential_type: "password".to_string(),
            algorithm: "plain".to_string(),
            salt_position: None,
            salt_rounds: None,
            mac_fun: None,
            iterations: None,
            dk_length: None,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            jwt_source: "password".to_string(),
            jwt_encryption: "hmac-based".to_string(),
            secret: None,
            secret_base64_encoded: Some(false),
            public_key: None,
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            method: "POST".to_string(),
            headers: None,
            body: None,
        }
    }
}
