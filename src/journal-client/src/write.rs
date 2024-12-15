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

use std::sync::Arc;

use common_base::error::common::CommonError;

use crate::async_writer::{AsyncWriter, SenderMessage, SenderMessageResp};
use crate::cache::{get_active_segment, MetadataCache};
use crate::connection::ConnectionManager;
use crate::error::JournalClientError;

#[derive(Default, Clone)]
pub struct JournalClientWriteData {
    pub key: String,
    pub content: Vec<u8>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct JournalWrite {
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    writer: Arc<AsyncWriter>,
}
impl JournalWrite {
    pub fn new(addrs: Vec<String>) -> Self {
        let metadata_cache = Arc::new(MetadataCache::new(addrs));
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));
        let writer = Arc::new(AsyncWriter::new(
            connection_manager.clone(),
            metadata_cache.clone(),
        ));
        JournalWrite {
            connection_manager,
            metadata_cache,
            writer,
        }
    }

    pub async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        data: Vec<JournalClientWriteData>,
    ) -> Result<Vec<SenderMessageResp>, JournalClientError> {
        let active_segment = get_active_segment(
            &self.metadata_cache,
            &self.connection_manager,
            &namespace,
            &shard_name,
        )
        .await;

        let message = SenderMessage::build(&namespace, &shard_name, active_segment, data.clone());
        self.writer.send(&message).await
    }

    pub async fn write(
        &self,
        namespace: String,
        shard_name: String,
        data: JournalClientWriteData,
    ) -> Result<SenderMessageResp, JournalClientError> {
        let resp_vec = self.batch_write(namespace, shard_name, vec![data]).await?;
        if let Some(resp) = resp_vec.first() {
            return Ok(resp.to_owned());
        }
        Err(JournalClientError::WriteReqReturnTmpty)
    }

    pub async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
