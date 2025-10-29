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
use std::path::Path;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LocalFileConnectorConfig {
    pub local_file_path: String,
}

impl LocalFileConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.local_file_path.is_empty() {
            return Err(CommonError::CommonError(
                "local_file_path cannot be empty".to_string(),
            ));
        }

        if self.local_file_path.len() > 4096 {
            return Err(CommonError::CommonError(
                "local_file_path length cannot exceed 4096 characters".to_string(),
            ));
        }

        let path = Path::new(&self.local_file_path);

        if self.local_file_path.contains('\0') {
            return Err(CommonError::CommonError(
                "local_file_path contains invalid null byte".to_string(),
            ));
        }

        if !path.is_absolute() {
            return Err(CommonError::CommonError(
                "local_file_path must be an absolute path (e.g., /var/log/mqtt.log or C:\\logs\\mqtt.log)".to_string(),
            ));
        }

        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() && path.has_root() {
            } else if parent.as_os_str().is_empty() {
                return Err(CommonError::CommonError(
                    "local_file_path must include a directory path".to_string(),
                ));
            }
        }

        if path.file_name().is_none() {
            return Err(CommonError::CommonError(
                "local_file_path must include a filename".to_string(),
            ));
        }

        for component in path.components() {
            let component_str = component.as_os_str().to_string_lossy();
            if component_str == ".." {
                return Err(CommonError::CommonError(
                    "local_file_path cannot contain '..' directory traversal".to_string(),
                ));
            }
        }

        Ok(())
    }
}
