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

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use common_base::tools::now_mills;
use dashmap::DashMap;
use log::{error, warn};
use metadata_struct::journal::segment::segment_name;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::{
    WriteReqBody, WriteReqMessages, WriteReqSegmentMessages,
};
use tokio::select;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::time::{sleep, timeout};

use crate::cache::{load_shards_cache, MetadataCache};
use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::batch_write;

// Send Message Struct
#[derive(Clone)]
pub struct SenderMessage {
    namespace: String,
    shard_name: String,
    segment: u32,
    key: String,
    content: Vec<u8>,
    tags: Vec<String>,
}

impl SenderMessage {
    pub fn build(
        namespace: &String,
        shard_name: &String,
        segment: u32,
        key: &String,
        content: &Vec<u8>,
        tags: &Vec<String>,
    ) -> Self {
        SenderMessage {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            segment,
            key: key.to_owned(),
            content: content.to_owned(),
            tags: tags.to_owned(),
        }
    }
}

// Send Message Resp Struct
#[derive(Clone)]
pub struct SenderMessageResp {
    offset: u64,
    error: Option<String>,
}

// Node Sender Threade Struct
#[derive(Clone)]
struct NodeSenderThread {
    node_send: Sender<DataSenderPkg>,
    stop_send: broadcast::Sender<bool>,
}

// Data Sender Pkg
#[derive(Clone)]
pub struct DataSenderPkg {
    callback_sx: Sender<SenderMessageResp>,
    message: SenderMessage,
}

// Data Sender
pub struct DataSender {
    node_senders: DashMap<u64, NodeSenderThread>,
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
}

impl DataSender {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        metadata_cache: Arc<MetadataCache>,
    ) -> Self {
        let node_senders = DashMap::with_capacity(2);
        DataSender {
            node_senders,
            connection_manager,
            metadata_cache,
        }
    }

    fn add_node_sender(&self, node_id: u64) -> Sender<DataSenderPkg> {
        let (data_sender, _) = broadcast::channel::<DataSenderPkg>(1000);
        let (stop_send, _) = broadcast::channel::<bool>(1);

        let node_sender_thread = NodeSenderThread {
            node_send: data_sender.clone(),
            stop_send: stop_send.clone(),
        };

        self.node_senders.insert(node_id, node_sender_thread);

        start_sender_thread(
            node_id,
            self.connection_manager.clone(),
            self.metadata_cache.clone(),
            data_sender.clone(),
            stop_send,
        );

        data_sender
    }

    pub fn remove_node_sender(&self, node_id: u64) {
        if let Some(node) = self.node_senders.get(&node_id) {
            if let Ok(e) = node.stop_send.send(true) {
                self.node_senders.remove(&node_id);
            }
        }
    }

    pub async fn send(
        &self,
        message: &SenderMessage,
    ) -> Result<SenderMessageResp, JournalClientError> {
        let active_segment_leader = if let Some(segment) = self
            .metadata_cache
            .get_segment_leader(&message.namespace, &message.shard_name)
        {
            segment
        } else {
            if let Err(e) = load_shards_cache(
                &self.metadata_cache,
                &self.connection_manager,
                &message.namespace,
                &message.shard_name,
            )
            .await
            {
                error!(
                    "Loading Shard {} Metadata info failed, error message :{}",
                    shard_name_iden(&message.namespace, &message.shard_name,),
                    e
                );
            }
            return Err(JournalClientError::NotActiveSegmentLeader(shard_name_iden(
                &message.namespace,
                &message.shard_name,
            )));
        };

        let sender = if let Some(sender) = self.node_senders.get(&active_segment_leader) {
            sender.node_send.clone()
        } else {
            self.add_node_sender(active_segment_leader)
        };

        let (callback_sx, mut callback_rx) = broadcast::channel::<SenderMessageResp>(2);
        let data = DataSenderPkg {
            callback_sx,
            message: message.to_owned(),
        };

        let rs = sender.send(data);
        let resp_data = timeout(Duration::from_secs(30), callback_rx.recv()).await?;
        Ok(resp_data?)
    }
}

pub fn start_sender_thread(
    node_id: u64,
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    node_send: Sender<DataSenderPkg>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut stop_recv = stop_send.subscribe();
        let mut node_recv = node_send.subscribe();
        let pkid_generator = AtomicU64::new(0);
        loop {
            select! {
                val = stop_recv.recv()=>{
                    if let Err(flag) = val {

                    }
                },
                messages = get_batch_message(&mut node_recv, 10)=>{
                    if messages.is_empty(){
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                    batch_sender_message(&connection_manager,&metadata_cache, node_id, &pkid_generator, messages).await;
                }
            }
        }
    });
}

async fn batch_sender_message(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    node_id: u64,
    pkid_generator: &AtomicU64,
    messages: Vec<DataSenderPkg>,
) {
    // build data by namespace&shard_name&segment
    let mut segment_data_list: HashMap<String, Vec<DataSenderPkg>> = HashMap::new();
    for pkg in messages.iter() {
        let key = segment_name(
            &pkg.message.namespace,
            &pkg.message.shard_name,
            pkg.message.segment,
        );
        if let Some(list) = segment_data_list.get_mut(&key) {
            list.push(pkg.to_owned());
        } else {
            let list = vec![pkg.to_owned()];
            segment_data_list.insert(key, list);
        }
    }

    // build WriteReqSegmentMessages
    let mut segments: Vec<WriteReqSegmentMessages> = Vec::new();
    let mut callback_sender = HashMap::new();
    for (_, messages) in segment_data_list {
        if messages.is_empty() {
            continue;
        }
        let first_msg = messages.first().unwrap();
        let namespace = first_msg.message.namespace.to_owned();
        let shard_name = first_msg.message.shard_name.to_owned();
        let segment = first_msg.message.segment;

        let mut write_req_segment_messages = Vec::new();
        for msg in messages {
            let pkid = pkid_generator.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            write_req_segment_messages.push(WriteReqMessages {
                pkid,
                key: msg.message.key,
                value: msg.message.content,
                tags: msg.message.tags,
            });
            callback_sender.insert(pkid, msg.callback_sx);
        }

        let msg = WriteReqSegmentMessages {
            namespace,
            shard_name,
            segment,
            messages: write_req_segment_messages,
        };
        segments.push(msg);
    }

    // send data
    let body = WriteReqBody { data: segments };
    match batch_write(connection_manager, node_id, body).await {
        Ok(data) => {
            // callback resp
            for shard_msg in data.status {
                for msg in shard_msg.messages {
                    if let Some(callback_sx) = callback_sender.get(&msg.pkid) {
                        let resp = if let Some(e) = msg.error {
                            SenderMessageResp {
                                offset: 0,
                                error: Some(format!("{}:{}", e.code, e.error)),
                            }
                        } else {
                            SenderMessageResp {
                                offset: msg.offset,
                                error: None,
                            }
                        };
                        if let Err(e) = callback_sx.send(resp) {
                            error!("{}", e);
                        }
                    } else {
                        warn!("");
                    }
                }
            }
        }
        Err(e) => {
            // callback error
            for (_, callback_sx) in callback_sender {
                if let Err(e) = callback_sx.send(SenderMessageResp {
                    offset: 0,
                    error: Some(e.to_string()),
                }) {
                    error!("{}", e);
                }
            }
        }
    }
}

// Fetching data in bulk
async fn get_batch_message(recv: &mut Receiver<DataSenderPkg>, line_ms: u64) -> Vec<DataSenderPkg> {
    let mut results = Vec::new();
    let start_ms = now_mills();
    loop {
        if (now_mills() - start_ms) > line_ms as u128 {
            break;
        }

        let data = timeout(Duration::from_millis(2), recv.recv()).await;
        if let Ok(Ok(data_1)) = data {
            results.push(data_1);
            continue;
        }
    }
    results
}
