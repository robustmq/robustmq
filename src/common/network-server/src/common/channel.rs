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

use crate::common::packet::RequestPackage;
use dashmap::DashMap;
use metadata_struct::connection::NetworkConnectionType;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::error;

#[derive(Clone)]
pub struct RequestChannel {
    pub handler_channels: DashMap<String, Sender<RequestPackage>>,
    pub channel_size: usize,
    pub req_packet_seq: Arc<AtomicU64>,
}

impl RequestChannel {
    pub fn new(channel_size: usize) -> Self {
        RequestChannel {
            handler_channels: DashMap::with_capacity(2),
            req_packet_seq: Arc::new(AtomicU64::new(0)),
            channel_size,
        }
    }

    pub fn create_handler_channel(
        &self,
        network_type: &NetworkConnectionType,
        index: usize,
    ) -> Receiver<RequestPackage> {
        let (sx, rx) = mpsc::channel::<RequestPackage>(self.channel_size);
        let key = self.key_name(network_type, index as u64);
        self.handler_channels.insert(key, sx);
        rx
    }

    pub async fn send_request_packet_to_handler(
        &self,
        network_type: &NetworkConnectionType,
        packet: RequestPackage,
    ) {
        let index = self.calc_req_channel_index();
        let key = self.key_name(network_type, index);
        if let Some(handler_sx) = self.handler_channels.get(&key) {
            if let Err(e) = handler_sx.send(packet).await {
                error!("Failed to send packet to handler {}: {}", key, e);
            }
        } else {
            error!("No handler channel available for {}", key);
        }
    }

    fn key_name(&self, network_type: &NetworkConnectionType, index: u64) -> String {
        format!("{network_type}_{index}")
    }

    pub fn calc_req_channel_index(&self) -> u64 {
        let index = self.req_packet_seq.fetch_add(1, Ordering::SeqCst);
        let seq = index % (self.handler_channels.len()) as u64;
        if seq == 0 {
            return 1;
        }
        seq
    }
}
