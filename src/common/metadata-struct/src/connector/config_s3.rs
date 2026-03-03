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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct S3ConnectorConfig {
    pub bucket: String,
    pub region: String,

    #[serde(default)]
    pub endpoint: String,

    #[serde(default)]
    pub access_key_id: String,

    #[serde(default)]
    pub secret_access_key: String,

    #[serde(default)]
    pub session_token: String,

    #[serde(default)]
    pub root: String,

    #[serde(default = "default_object_key_prefix")]
    pub object_key_prefix: String,

    #[serde(default = "default_file_extension")]
    pub file_extension: String,
}

fn default_object_key_prefix() -> String {
    "mqtt".to_string()
}

fn default_file_extension() -> String {
    "json".to_string()
}

impl Default for S3ConnectorConfig {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: String::new(),
            endpoint: String::new(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            session_token: String::new(),
            root: String::new(),
            object_key_prefix: default_object_key_prefix(),
            file_extension: default_file_extension(),
        }
    }
}

impl S3ConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.bucket.is_empty() {
            return Err(CommonError::CommonError(
                "bucket cannot be empty".to_string(),
            ));
        }

        if self.bucket.len() > 63 {
            return Err(CommonError::CommonError(
                "bucket length cannot exceed 63 characters".to_string(),
            ));
        }

        if self.region.is_empty() {
            return Err(CommonError::CommonError(
                "region cannot be empty".to_string(),
            ));
        }

        if self.region.len() > 128 {
            return Err(CommonError::CommonError(
                "region length cannot exceed 128 characters".to_string(),
            ));
        }

        if !self.endpoint.is_empty()
            && !self.endpoint.starts_with("http://")
            && !self.endpoint.starts_with("https://")
        {
            return Err(CommonError::CommonError(
                "endpoint must start with http:// or https://".to_string(),
            ));
        }

        if self.root.len() > 256 {
            return Err(CommonError::CommonError(
                "root length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.object_key_prefix.trim().is_empty() {
            return Err(CommonError::CommonError(
                "object_key_prefix cannot be empty".to_string(),
            ));
        }

        if self.object_key_prefix.len() > 256 {
            return Err(CommonError::CommonError(
                "object_key_prefix length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.file_extension.trim().is_empty() {
            return Err(CommonError::CommonError(
                "file_extension cannot be empty".to_string(),
            ));
        }

        if self.file_extension.len() > 16 {
            return Err(CommonError::CommonError(
                "file_extension length cannot exceed 16 characters".to_string(),
            ));
        }

        if !self
            .file_extension
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
        {
            return Err(CommonError::CommonError(
                "file_extension must be alphanumeric".to_string(),
            ));
        }

        let has_access_key = !self.access_key_id.is_empty();
        let has_secret_key = !self.secret_access_key.is_empty();
        if has_access_key != has_secret_key {
            return Err(CommonError::CommonError(
                "access_key_id and secret_access_key must be set together".to_string(),
            ));
        }

        Ok(())
    }
}
