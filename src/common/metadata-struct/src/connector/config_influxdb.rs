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

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InfluxDBVersion {
    V1,
    #[default]
    V2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InfluxDBPrecision {
    #[serde(rename = "ns")]
    Nanoseconds,
    #[serde(rename = "us")]
    Microseconds,
    #[default]
    #[serde(rename = "ms")]
    Milliseconds,
    #[serde(rename = "s")]
    Seconds,
}

impl InfluxDBPrecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            InfluxDBPrecision::Nanoseconds => "ns",
            InfluxDBPrecision::Microseconds => "us",
            InfluxDBPrecision::Milliseconds => "ms",
            InfluxDBPrecision::Seconds => "s",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InfluxDBConnectorConfig {
    pub server: String,
    #[serde(default)]
    pub version: InfluxDBVersion,

    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub org: String,
    #[serde(default)]
    pub bucket: String,

    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,

    pub measurement: String,

    #[serde(default)]
    pub precision: InfluxDBPrecision,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    15
}

impl Default for InfluxDBConnectorConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            version: InfluxDBVersion::default(),
            token: String::new(),
            org: String::new(),
            bucket: String::new(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            measurement: String::new(),
            precision: InfluxDBPrecision::default(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl InfluxDBConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.server.is_empty() {
            return Err(CommonError::CommonError(
                "server cannot be empty".to_string(),
            ));
        }

        if !self.server.starts_with("http://") && !self.server.starts_with("https://") {
            return Err(CommonError::CommonError(
                "server must start with http:// or https://".to_string(),
            ));
        }

        if self.measurement.is_empty() {
            return Err(CommonError::CommonError(
                "measurement cannot be empty".to_string(),
            ));
        }

        match self.version {
            InfluxDBVersion::V2 => {
                if self.token.is_empty() {
                    return Err(CommonError::CommonError(
                        "token is required for InfluxDB v2".to_string(),
                    ));
                }
                if self.org.is_empty() {
                    return Err(CommonError::CommonError(
                        "org is required for InfluxDB v2".to_string(),
                    ));
                }
                if self.bucket.is_empty() {
                    return Err(CommonError::CommonError(
                        "bucket is required for InfluxDB v2".to_string(),
                    ));
                }
            }
            InfluxDBVersion::V1 => {
                if self.database.is_empty() {
                    return Err(CommonError::CommonError(
                        "database is required for InfluxDB v1".to_string(),
                    ));
                }
            }
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        Ok(())
    }

    pub fn write_url(&self) -> String {
        let base = self.server.trim_end_matches('/');
        match self.version {
            InfluxDBVersion::V2 => {
                format!(
                    "{}/api/v2/write?org={}&bucket={}&precision={}",
                    base,
                    self.org,
                    self.bucket,
                    self.precision.as_str()
                )
            }
            InfluxDBVersion::V1 => {
                let mut url = format!(
                    "{}/write?db={}&precision={}",
                    base,
                    self.database,
                    self.precision.as_str()
                );
                if !self.username.is_empty() {
                    url.push_str(&format!("&u={}", self.username));
                }
                if !self.password.is_empty() {
                    url.push_str(&format!("&p={}", self.password));
                }
                url
            }
        }
    }
}
