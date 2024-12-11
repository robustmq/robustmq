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
use log::error;
use metadata_struct::journal::segment::segment_name;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::{
    WriteReqBody, WriteReqMessages, WriteReqSegmentMessages,
};
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{sleep, timeout};

use crate::cache::{load_shards_cache, MetadataCache};
use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::batch_write;
use crate::JournalClientWriteData;

// Send Message Struct
#[derive(Clone)]
pub struct SenderMessage {
    namespace: String,
    shard_name: String,
    segment: u32,
    data: Vec<JournalClientWriteData>,
}

impl SenderMessage {
    pub fn build(
        namespace: &String,
        shard_name: &String,
        segment: u32,
        data: Vec<JournalClientWriteData>,
    ) -> Self {
        SenderMessage {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            segment,
            data,
        }
    }
}

// Send Message Resp Struct
#[derive(Clone, Default)]
pub struct SenderMessageResp {
    pub offset: u64,
    pub error: Option<String>,
}

// Node Sender Threade Struct
#[derive(Clone)]
struct NodeSenderThread {
    node_send: Sender<DataSenderPkg>,
    stop_send: mpsc::Sender<bool>,
}

// Data Sender Pkg
#[derive(Clone)]
pub struct DataSenderPkg {
    sender_pkg_id: u64,
    callback_sx: Sender<Vec<SenderMessageResp>>,
    message: SenderMessage,
}

// Data Sender
pub struct Writer {
    node_senders: DashMap<u64, NodeSenderThread>,
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    send_pkid_generator: AtomicU64,
}

impl Writer {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        metadata_cache: Arc<MetadataCache>,
    ) -> Self {
        let node_senders = DashMap::with_capacity(2);
        let send_pkid_generator = AtomicU64::new(0);
        Writer {
            node_senders,
            connection_manager,
            metadata_cache,
            send_pkid_generator,
        }
    }

    fn add_node_sender(&self, node_id: u64) -> Sender<DataSenderPkg> {
        let (data_sender, data_recv) = mpsc::channel::<DataSenderPkg>(1000);
        let (stop_send, stop_recv) = mpsc::channel::<bool>(1);

        let node_sender_thread = NodeSenderThread {
            node_send: data_sender.clone(),
            stop_send: stop_send.clone(),
        };

        self.node_senders.insert(node_id, node_sender_thread);

        start_sender_thread(
            node_id,
            self.connection_manager.clone(),
            self.metadata_cache.clone(),
            data_recv,
            stop_recv,
        );

        data_sender
    }

    pub async fn remove_node_sender(&self, node_id: u64) {
        if let Some(node) = self.node_senders.get(&node_id) {
            if let Ok(e) = node.stop_send.send(true).await {
                self.node_senders.remove(&node_id);
            }
        }
    }

    pub async fn send(
        &self,
        message: &SenderMessage,
    ) -> Result<Vec<SenderMessageResp>, JournalClientError> {
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

        let (callback_sx, mut callback_rx) = mpsc::channel::<Vec<SenderMessageResp>>(2);
        let data = DataSenderPkg {
            sender_pkg_id: self
                .send_pkid_generator
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            callback_sx,
            message: message.to_owned(),
        };

        let rs = sender.send(data).await;
        let resp_data = timeout(Duration::from_secs(30), callback_rx.recv()).await?;
        Ok(resp_data.unwrap())
    }

    pub async fn close(&self) -> Result<(), JournalClientError> {
        for raw in self.node_senders.iter() {
            raw.value().stop_send.send(true).await?;
        }
        Ok(())
    }
}

pub fn start_sender_thread(
    node_id: u64,
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    mut node_recv: Receiver<DataSenderPkg>,
    mut stop_recv: Receiver<bool>,
) {
    tokio::spawn(async move {
        let pkid_generator = AtomicU64::new(0);

        loop {
            select! {
                val = stop_recv.recv()=>{
                    if let Some(flag) = val {

                    }
                },
                messages = get_batch_message(&mut node_recv, 100)=>{
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
    let (segments, data_pkgs, callback_sx) = build_send_data(pkid_generator, messages);

    // send data
    let body = WriteReqBody { data: segments };
    match batch_write(connection_manager, node_id, body).await {
        Ok(data) => {
            // callback resp
            let mut pkid_resp = HashMap::new();
            for shard_msg in data.status {
                for msg in shard_msg.messages {
                    let resp = if let Some(e) = msg.error {
                        SenderMessageResp {
                            error: Some(format!("{}:{}", e.code, e.error)),
                            ..Default::default()
                        }
                    } else {
                        SenderMessageResp {
                            offset: msg.offset,
                            error: None,
                        }
                    };
                    pkid_resp.insert(msg.pkid, resp);
                }
            }

            for (id, pkids) in data_pkgs {
                let mut results = Vec::new();
                for pkid in pkids {
                    if let Some(resp) = pkid_resp.get(&pkid) {
                        results.push(resp.to_owned());
                    }
                }

                if let Some(sx) = callback_sx.get(&id) {
                    if let Err(e) = sx.send(results).await {
                        error!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            // callback error
            for (id, pkids) in data_pkgs {
                let mut results = Vec::new();
                for pkid in pkids {
                    results.push(SenderMessageResp {
                        error: Some(e.to_string()),
                        ..Default::default()
                    });
                }

                if let Some(sx) = callback_sx.get(&id) {
                    if let Err(e) = sx.send(results).await {
                        error!("{}", e);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn build_send_data(
    pkid_generator: &AtomicU64,
    messages: Vec<DataSenderPkg>,
) -> (
    Vec<WriteReqSegmentMessages>,
    DashMap<u64, Vec<u64>>,
    DashMap<u64, Sender<Vec<SenderMessageResp>>>,
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
    let data_pkgs: DashMap<u64, Vec<u64>> = DashMap::with_capacity(2);
    let callback_sx: DashMap<u64, Sender<Vec<SenderMessageResp>>> = DashMap::with_capacity(2);
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
            for (i, raw) in msg.message.data.iter().enumerate() {
                let pkid = pkid_generator.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                write_req_segment_messages.push(WriteReqMessages {
                    pkid,
                    key: raw.key.to_owned(),
                    value: raw.content.to_owned(),
                    tags: raw.tags.to_owned(),
                });
                if let Some(mut data_mut) = data_pkgs.get_mut(&msg.sender_pkg_id) {
                    data_mut.push(pkid);
                } else {
                    data_pkgs.insert(msg.sender_pkg_id, vec![pkid]);
                }
            }
            callback_sx.insert(msg.sender_pkg_id, msg.callback_sx);
        }

        let msg = WriteReqSegmentMessages {
            namespace,
            shard_name,
            segment,
            messages: write_req_segment_messages,
        };
        segments.push(msg);
    }
    (segments, data_pkgs, callback_sx)
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
        if let Ok(Some(data_1)) = data {
            results.push(data_1);
            continue;
        }
    }
    results
}
