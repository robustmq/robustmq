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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;
use crate::{
    common::error::Result,
};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogOffsetMetadata {
    pub message_offset: i64,
    pub segment_start_offset: i64,
}

pub struct OffsetManager {
    store_cache: Arc<Mutex<HashMap<(String, i32), LogOffsetMetadata>>>,
}

impl OffsetManager {

    pub fn new() -> Self {
        Self {
            store_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn next_offset(&self, topic: &str, partition: i32) -> Result<i64> {
        let mut cache = self.store_cache.lock().await;
        let key = (topic.to_string(), partition);

        let offsets = cache.entry(key.clone()).or_insert_with(|| {
            LogOffsetMetadata::default()
        });

        let next_offset = offsets.next_offset;
        offsets.next_offset += 1;

        if next_offset % 100 == 0 {
            let topic_clone = topic.to_string();
            tokio::spawn(async move {
                if let Err(e) = Self::persist_offsets(
                    self,
                    &topic_clone,
                    partition,
                    0,
                ).await {
                    debug!("Failed to persist offsets for topic {} partition {}: {:?}", topic_clone, partition, e);
                }
            });
        }
        Ok(next_offset)
    }

    pub async fn persist_offsets(
        &self, topic: &str, partition: i32, offset: i64) -> Result<()> {
        Ok(())
    }
}