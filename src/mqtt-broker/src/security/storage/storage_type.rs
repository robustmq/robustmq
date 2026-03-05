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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum AuthDataStorageType {
    #[default]
    Meta,
    Mysql,
    Postgresql,
    Redis,
    Mongodb,
    Http,
}

impl FromStr for AuthDataStorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "meta" => Ok(AuthDataStorageType::Meta),
            "mysql" => Ok(AuthDataStorageType::Mysql),
            "postgresql" => Ok(AuthDataStorageType::Postgresql),
            "redis" => Ok(AuthDataStorageType::Redis),
            "mongodb" => Ok(AuthDataStorageType::Mongodb),
            "http" => Ok(AuthDataStorageType::Http),
            _ => Err(format!("invalid auth type: {s}")),
        }
    }
}
