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

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct OpenTSDBConnectorConfig {
    pub server: String,
    #[serde(default = "default_metric_field")]
    pub metric_field: String,
    #[serde(default = "default_value_field")]
    pub value_field: String,
    #[serde(default)]
    pub tags_fields: Vec<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub summary: bool,
    #[serde(default)]
    pub details: bool,
}

fn default_metric_field() -> String {
    "metric".to_string()
}

fn default_value_field() -> String {
    "value".to_string()
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

impl OpenTSDBConnectorConfig {
    pub fn validate(&self) -> Result<(), common_base::error::common::CommonError> {
        use common_base::error::common::CommonError;

        if self.server.is_empty() {
            return Err(CommonError::CommonError(
                "server cannot be empty".to_string(),
            ));
        }

        if self.server.len() > 512 {
            return Err(CommonError::CommonError(
                "server length cannot exceed 512 characters".to_string(),
            ));
        }

        if !self.server.starts_with("http://") && !self.server.starts_with("https://") {
            return Err(CommonError::CommonError(
                "server must start with http:// or https://".to_string(),
            ));
        }

        if self.metric_field.is_empty() {
            return Err(CommonError::CommonError(
                "metric_field cannot be empty".to_string(),
            ));
        }

        if self.value_field.is_empty() {
            return Err(CommonError::CommonError(
                "value_field cannot be empty".to_string(),
            ));
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        if self.max_retries > 10 {
            return Err(CommonError::CommonError(
                "max_retries cannot exceed 10".to_string(),
            ));
        }

        Ok(())
    }

    pub fn put_url(&self) -> String {
        let base = self.server.trim_end_matches('/');
        let mut url = format!("{}/api/put", base);
        let mut params = Vec::new();
        if self.summary {
            params.push("summary");
        }
        if self.details {
            params.push("details");
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        url
    }
}
