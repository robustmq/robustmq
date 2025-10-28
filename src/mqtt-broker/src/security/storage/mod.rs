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

use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub mod http;
pub mod mysql;
pub mod placement;
pub mod postgresql;
pub mod redis;
pub mod storage_trait;
pub mod sync;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum AuthType {
    #[default]
    Placement,
    Journal,
    Mysql,
    Postgresql,
    Redis,
    Jwt,
    Http,
}

impl FromStr for AuthType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "placement" => Ok(AuthType::Placement),
            "journal" => Ok(AuthType::Journal),
            "mysql" => Ok(AuthType::Mysql),
            "postgresql" => Ok(AuthType::Postgresql),
            "redis" => Ok(AuthType::Redis),
            "jwt" => Ok(AuthType::Jwt),
            "http" => Ok(AuthType::Http),
            _ => Err(format!("invalid auth type: {s}")),
        }
    }
}
