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

use crate::core::cache::QosAckPacketInfo;
use common_base::tools::now_second;
use dashmap::DashMap;
use protocol::mqtt::common::QoS;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::sleep;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PkidAckEnum {
    PubAck,
    PubRec,
    PubComp,
    SubAck,
    UnSubAck,
}

#[derive(Clone)]
pub struct ReceiveQosPkidData {
    pub ack_enum: PkidAckEnum,
    pub pkid: u16,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct PkidManager {
    // (client_id, (pkid, ReceiveQosPkidData))
    pub receive_pkid_data: DashMap<String, DashMap<u64, ReceiveQosPkidData>>,

    // publish to client pkid generate
    pub publish_to_client_pkid_generate: Arc<AtomicU64>,

    //(client_id, now_second())
    pub publish_to_client_pkid_cache: DashMap<String, DashMap<String, u64>>,

    //(client_id_pkid, AckPacketInfo)
    pub publish_to_client_qos_ack_data: DashMap<String, QosAckPacketInfo>,
}

impl Default for PkidManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PkidManager {
    pub fn new() -> Self {
        PkidManager {
            receive_pkid_data: DashMap::with_capacity(8),

            publish_to_client_pkid_generate: Arc::new(AtomicU64::new(1)),
            publish_to_client_pkid_cache: DashMap::with_capacity(8),
            publish_to_client_qos_ack_data: DashMap::with_capacity(8),
        }
    }
    // receive publish pkid data
    pub fn add_receive_publish_pkid_data(&self, client_id: &str, data: ReceiveQosPkidData) {
        let inner = self
            .receive_pkid_data
            .entry(client_id.to_string())
            .or_default();
        inner.insert(data.pkid as u64, data);
    }

    pub fn remove_receive_publish_pkid_data(&self, client_id: &str, pkid: u16) {
        let pkid_key = pkid as u64;
        let mut remove_outer = false;
        if let Some(inner) = self.receive_pkid_data.get(client_id) {
            inner.remove(&pkid_key);
            if inner.is_empty() {
                remove_outer = true;
            }
        }
        if remove_outer {
            self.receive_pkid_data.remove(client_id);
        }
    }

    pub fn get_receive_publish_pkid_data(
        &self,
        client_id: &str,
        pkid: u16,
    ) -> Option<ReceiveQosPkidData> {
        if let Some(inner) = self.receive_pkid_data.get(client_id) {
            if let Some(da) = inner.get(&(pkid as u64)) {
                return Some(da.clone());
            }
        }
        None
    }

    pub fn get_receive_publish_pkid_data_len_by_client_ids(&self, client_id: &str) -> usize {
        if let Some(inner) = self.receive_pkid_data.get(client_id) {
            return inner.len();
        }
        0
    }

    // publish to client pkid
    pub async fn generate_publish_to_client_pkid(&self, client_id: &str, qos: &QoS) -> u16 {
        if *qos == QoS::AtMostOnce {
            return 1;
        }

        loop {
            let seq = self
                .publish_to_client_pkid_generate
                .fetch_add(1, Ordering::SeqCst);

            let id = (seq % 65535) as u16;
            if id == 0 {
                sleep(Duration::from_millis(1)).await;
                continue;
            }

            let pkid_key = id.to_string();
            let should_retry = {
                let inner = self
                    .publish_to_client_pkid_cache
                    .entry(client_id.to_string())
                    .or_default();
                if inner.contains_key(&pkid_key) {
                    true
                } else {
                    inner.insert(pkid_key, now_second());
                    false
                }
            };
            if should_retry {
                sleep(Duration::from_millis(1)).await;
                continue;
            }

            return id;
        }
    }

    pub fn remove_publish_to_client_pkid(&self, client_id: &str, pkid: u16) {
        let pkid_key = pkid.to_string();
        let mut remove_outer = false;
        if let Some(inner) = self.publish_to_client_pkid_cache.get(client_id) {
            inner.remove(&pkid_key);
            if inner.is_empty() {
                remove_outer = true;
            }
        }
        if remove_outer {
            self.publish_to_client_pkid_cache.remove(client_id);
        }

        self.remove_publish_to_client_qos_ack_data(client_id, pkid);
    }

    // publish to client qos ack data
    pub fn add_publish_to_client_qos_ack_data(
        &self,
        client_id: &str,
        pkid: u16,
        packet: QosAckPacketInfo,
    ) {
        let key = self.key(client_id, pkid);
        self.publish_to_client_qos_ack_data.insert(key, packet);
    }

    pub fn get_publish_to_client_qos_ack_data(
        &self,
        client_id: &str,
        pkid: u16,
    ) -> Option<QosAckPacketInfo> {
        let key = self.key(client_id, pkid);
        if let Some(data) = self.publish_to_client_qos_ack_data.get(&key) {
            return Some(data.clone());
        }
        None
    }

    pub fn remove_publish_to_client_qos_ack_data(&self, client_id: &str, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.publish_to_client_qos_ack_data.remove(&key);
    }

    //
    pub fn remove_by_client_id(&self, _client_id: &str) {}

    fn key(&self, client_id: &str, pkid: u16) -> String {
        format!("{client_id}_{pkid}")
    }
}
