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
use std::fmt::Display;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GreptimeDBConnectorConfig {
    pub server_addr: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub precision: TimePrecision,
}

impl GreptimeDBConnectorConfig {
    /// create a new GreptimeDBConnectorConfig.
    /// note: currently we only support second precision, see [`Record`],
    /// **If you want to support other precision, you need to modify the [`Record`]'s timestaomp at the same time**
    ///
    /// [`Record`]: use crate::adapter::record::Record;
    pub fn new(server_addr: String, database: String, user: String, password: String) -> Self {
        GreptimeDBConnectorConfig {
            server_addr,
            database,
            user,
            password,
            precision: TimePrecision::Second,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub enum TimePrecision {
    #[default]
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl Display for TimePrecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePrecision::Second => write!(f, "s"),
            TimePrecision::Millisecond => write!(f, "ms"),
            TimePrecision::Microsecond => write!(f, "us"),
            TimePrecision::Nanosecond => write!(f, "ns"),
        }
    }
}
