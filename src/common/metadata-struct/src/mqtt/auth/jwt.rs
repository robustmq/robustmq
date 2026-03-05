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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JwtConfig {
    pub jwt_source: String,                  // password/username
    pub jwt_encryption: String,              // hmac-based/public-key
    pub secret: Option<String>,              // hmac-based need
    pub secret_base64_encoded: Option<bool>, // hmac-based need
    pub public_key: Option<String>,          // public-key need
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
