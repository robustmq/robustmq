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

use common_base::tools::now_mills;
use metadata_struct::adapter::read_config::ReadConfig;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::connection::ConnectionManager;
use crate::error::JournalClientError;

pub struct ReadShardByOffset {
    pub namespace: String,
    pub shard_name: String,
    pub offset: u64,
}

pub struct ReadMessageData {
    pub namespace: String,
    pub shard_name: String,
    pub segment: u32,
    pub offset: u64,
    pub key: String,
    pub value: Vec<u8>,
    pub tads: Vec<String>,
}

pub struct Reader {
    connection_manager: Arc<ConnectionManager>,
    data_sender: Sender<ReadMessageData>,
    data_recv: Receiver<ReadMessageData>,
    group_name: String,
}

impl Reader {
    pub fn new(connection_manager: Arc<ConnectionManager>, group_name: String) -> Self {
        let (data_sender, data_recv) = mpsc::channel::<ReadMessageData>(1000);
        Reader {
            connection_manager,
            data_sender,
            data_recv,
            group_name,
        }
    }

    pub async fn start_read_by_offset(
        &self,
        shards: Vec<ReadShardByOffset>,
        read_config: ReadConfig,
    ) {
        start_read_thread_by_group(self.connection_manager.clone(), self.data_sender.clone());
    }

    pub async fn read(&mut self, group: &str) -> Result<Vec<ReadMessageData>, JournalClientError> {
        let mut results: Vec<ReadMessageData> = Vec::new();
        let start_time = now_mills();
        loop {
            if (now_mills() - start_time) >= 100 {
                break;
            }
            if let Some(data) = self.data_recv.recv().await {
                results.push(data);
            }
        }
        Ok(results)
    }
}

pub fn start_read_thread_by_group(
    connection_manager: Arc<ConnectionManager>,
    data_sender: Sender<ReadMessageData>,
) {
    tokio::spawn(async move {});
}
