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
    _strategy: MessageExpireStrategy,
    // data_size: Option<u32>,
    _timestamp: Option<u32>,
}

#[derive(Default, Clone)]
pub enum MessageExpireStrategy {
    #[default]
    DataSize,
    // Timestamp,
}

pub fn message_expire_thread(driver: ArcStorageAdapter, config: MessageExpireConfig) {
    tokio::spawn(async move {
        if let Err(e) = driver.message_expire(&config).await {
            error!("{}", e);
        }
    });
}
