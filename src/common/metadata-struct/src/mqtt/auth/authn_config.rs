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

use super::jwt::JwtConfig;
use super::password::PasswordBasedConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthnConfig {
    pub uid: String,
    pub authn_type: String, // Password-Based/JWT/SCRAM/GSSAPI/ClientInfo...
    pub config: LoginAuthEnum,
    pub create_at: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum LoginAuthEnum {
    PasswordBased(Box<PasswordBasedConfig>),
    JWT(JwtConfig),
}
