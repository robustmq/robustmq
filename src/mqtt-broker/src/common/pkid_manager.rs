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

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use common_base::tools::now_second;
use dashmap::DashMap;
use protocol::mqtt::common::QoS;
use tokio::time::sleep;

use crate::handler::cache::{ClientPkidData, QosAckPacketInfo};

#[derive(Clone)]
pub struct PkidManager {
    //(client_id_pkid, u64)
    pub pkid_cache: DashMap<String, u64>,

    //(client_id_pkid, AckPacketInfo)
    pub qos_ack_packet: DashMap<String, QosAckPacketInfo>,

    // (client_id_pkid, QosPkidData)
    pub client_pkid_data: DashMap<String, ClientPkidData>,

    pub pkid_atomic: Arc<AtomicU64>,
}

impl Default for PkidManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PkidManager {
    pub fn new() -> Self {
        PkidManager {
            pkid_cache: DashMap::with_capacity(8),
            qos_ack_packet: DashMap::with_capacity(8),
            client_pkid_data: DashMap::with_capacity(8),
            pkid_atomic: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn remove_by_client_id(&self, client_id: &str) {
        for (key, _) in self.qos_ack_packet.clone() {
            if key.starts_with(client_id) {
                self.qos_ack_packet.remove(&key);
            }
        }

        for (key, _) in self.client_pkid_data.clone() {
            if key.starts_with(client_id) {
                self.qos_ack_packet.remove(&key);
            }
        }
    }

    // sub => pub push pkid generate
    pub async fn generate_pkid(&self, client_id: &str, qos: &QoS) -> u16 {
        if *qos == QoS::AtMostOnce {
            return 1;
        }

        loop {
            let seq = self.pkid_atomic.fetch_add(1, Ordering::SeqCst);

            let id = (seq % 65535) as u16;
            if id == 0 {
                sleep(Duration::from_millis(1)).await;
                continue;
            }

            let key = self.key(client_id, id);
            if self.pkid_cache.contains_key(&key) {
                sleep(Duration::from_millis(1)).await;
                continue;
            }
            self.pkid_cache.insert(key, now_second());
            return id;
        }
    }

    // ack packet
    pub fn remove_ack_packet(&self, client_id: &str, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.qos_ack_packet.remove(&key);
        self.pkid_cache.remove(&key);
    }

    pub fn add_ack_packet(&self, client_id: &str, pkid: u16, packet: QosAckPacketInfo) {
        let key = self.key(client_id, pkid);
        self.qos_ack_packet.insert(key, packet);
    }

    pub fn get_ack_packet(&self, client_id: &str, pkid: u16) -> Option<QosAckPacketInfo> {
        let key = self.key(client_id, pkid);
        if let Some(data) = self.qos_ack_packet.get(&key) {
            return Some(data.clone());
        }
        None
    }

    // client pkid
    pub fn add_client_pkid(&self, client_id: &str, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.client_pkid_data.insert(
            key,
            ClientPkidData {
                client_id: client_id.to_owned(),
                create_time: now_second(),
            },
        );
    }

    pub fn delete_client_pkid(&self, client_id: &str, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.client_pkid_data.remove(&key);
    }

    pub fn get_client_pkid(&self, client_id: &str, pkid: u16) -> Option<ClientPkidData> {
        let key = self.key(client_id, pkid);
        if let Some(data) = self.client_pkid_data.get(&key) {
            return Some(data.clone());
        }
        None
    }

    fn key(&self, client_id: &str, pkid: u16) -> String {
        format!("{client_id}_{pkid}")
    }
}
