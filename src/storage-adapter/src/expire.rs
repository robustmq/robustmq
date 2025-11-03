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

use tracing::error;

use crate::storage::ArcStorageAdapter;

#[derive(Default, Clone)]
pub struct MessageExpireConfig {
    // data_size: Option<u32>,
    timestamp: Option<u32>,
}

impl MessageExpireConfig {
    /// Get the retention period in seconds
    pub fn get_timestamp(&self) -> Option<u32> {
        self.timestamp
    }

    /// Set the retention period in seconds
    pub fn set_timestamp(&mut self, timestamp: Option<u32>) {
        self.timestamp = timestamp;
    }

    /// Create a new config with timestamp retention
    pub fn with_timestamp(timestamp: u32) -> Self {
        Self {
            timestamp: Some(timestamp),
        }
    }
}

#[derive(Default, Clone)]
pub enum MessageExpireStrategy {
    #[default]
    // DataSize,
    Timestamp,
}

pub async fn message_expire_thread(driver: ArcStorageAdapter, config: MessageExpireConfig) {
    if let Err(e) = driver.message_expire(&config).await {
        error!("{}", e);
    }
}
