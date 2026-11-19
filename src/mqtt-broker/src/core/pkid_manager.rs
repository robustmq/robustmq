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
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_millis, now_second};
use dashmap::DashMap;
use protocol::mqtt::common::QoS;
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PkidAckEnum {
    PubAck,
    PubRec,
    PubComp,
    SubAck,
    UnSubAck,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveQosPkidData {
    pub ack_enum: PkidAckEnum,
    pub pkid: u16,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct PkidManager {
    // (client_id, (pkid, ReceiveQosPkidData)) — stores both QoS1 and QoS2 in-flight pkid state
    pub qos_pkid_data: DashMap<String, DashMap<u64, ReceiveQosPkidData>>,

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
            qos_pkid_data: DashMap::with_capacity(8),

            publish_to_client_pkid_generate: Arc::new(AtomicU64::new(1)),
            publish_to_client_pkid_cache: DashMap::with_capacity(8),
            publish_to_client_qos_ack_data: DashMap::with_capacity(8),
        }
    }

    // qos pkid data (shared for QoS1 and QoS2)
    pub fn add_qos_pkid_data(&self, client_id: &str, data: ReceiveQosPkidData) {
        let inner = self.qos_pkid_data.entry(client_id.to_string()).or_default();
        inner.insert(data.pkid as u64, data);
    }

    pub fn remove_qos_pkid_data(&self, client_id: &str, pkid: u16) {
        let pkid_key = pkid as u64;
        let mut remove_outer = false;
        if let Some(inner) = self.qos_pkid_data.get(client_id) {
            inner.remove(&pkid_key);
            if inner.is_empty() {
                remove_outer = true;
            }
        }
        if remove_outer {
            self.qos_pkid_data.remove(client_id);
        }
    }

    pub fn get_qos_pkid_data(&self, client_id: &str, pkid: u16) -> Option<ReceiveQosPkidData> {
        if let Some(inner) = self.qos_pkid_data.get(client_id) {
            if let Some(da) = inner.get(&(pkid as u64)) {
                return Some(da.clone());
            }
        }
        None
    }

    pub fn get_qos_pkid_data_len_by_client_id(&self, client_id: &str) -> usize {
        if let Some(inner) = self.qos_pkid_data.get(client_id) {
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

    pub fn remove_by_client_id(&self, client_id: &str) {
        self.qos_pkid_data.remove(client_id);
        self.publish_to_client_pkid_cache.remove(client_id);
        let prefix = format!("{client_id}_");
        self.publish_to_client_qos_ack_data
            .retain(|k, _| !k.starts_with(&prefix));
    }

    fn key(&self, client_id: &str, pkid: u16) -> String {
        format!("{client_id}_{pkid}")
    }

    pub fn clean_expired_pkid_data(&self, expire_secs: u64) {
        let now_sec = now_second();
        let now_ms = now_millis();
        let expire_ms = expire_secs as u128 * 1000;

        self.qos_pkid_data.retain(|_, inner| {
            inner.retain(|_, v| now_sec.saturating_sub(v.create_time) < expire_secs);
            !inner.is_empty()
        });

        self.publish_to_client_pkid_cache.retain(|_, inner| {
            inner.retain(|_, ts| now_sec.saturating_sub(*ts) < expire_secs);
            !inner.is_empty()
        });

        self.publish_to_client_qos_ack_data
            .retain(|_, v| now_ms.saturating_sub(v.create_time) < expire_ms);
    }
}

const PKID_CLEAN_EXPIRE_SECS: u64 = 60;
const PKID_CLEAN_INTERVAL_MS: u64 = 60_000;

pub async fn clean_pkid_data(
    cache_manager: Arc<crate::core::cache::MQTTCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        cache_manager
            .pkid_manager
            .clean_expired_pkid_data(PKID_CLEAN_EXPIRE_SECS);
        Ok(())
    };

    loop_select_ticket(ac_fn, PKID_CLEAN_INTERVAL_MS, &stop_send).await;
}

#[cfg(test)]
mod tests {
    use super::{PkidAckEnum, PkidManager, ReceiveQosPkidData};
    use crate::core::cache::QosAckPacketInfo;
    use common_base::tools::{now_millis, now_second};
    use protocol::mqtt::common::QoS;
    use std::sync::atomic::Ordering;
    use tokio::sync::mpsc;

    fn qos_data(pkid: u16, create_time: u64) -> ReceiveQosPkidData {
        ReceiveQosPkidData {
            ack_enum: PkidAckEnum::PubAck,
            pkid,
            create_time,
        }
    }

    fn ack_data(create_time: u128) -> QosAckPacketInfo {
        let (sx, _rx) = mpsc::channel(1);
        QosAckPacketInfo { sx, create_time }
    }

    #[test]
    fn qos_packet_data_supports_add_get_remove_and_client_cleanup() {
        let manager = PkidManager::new();
        manager.add_qos_pkid_data("client-1", qos_data(7, 10));
        manager.add_qos_pkid_data("client-1", qos_data(8, 11));

        assert_eq!(manager.get_qos_pkid_data_len_by_client_id("client-1"), 2);
        assert_eq!(manager.get_qos_pkid_data("client-1", 7).unwrap().pkid, 7);
        assert!(manager.get_qos_pkid_data("missing", 7).is_none());

        manager.remove_qos_pkid_data("client-1", 7);
        assert!(manager.get_qos_pkid_data("client-1", 7).is_none());
        assert_eq!(manager.get_qos_pkid_data_len_by_client_id("client-1"), 1);

        manager.remove_by_client_id("client-1");
        assert_eq!(manager.get_qos_pkid_data_len_by_client_id("client-1"), 0);
    }

    #[tokio::test]
    async fn publish_packet_ids_are_qos_aware_and_skip_zero_or_collisions() {
        let manager = PkidManager::new();
        assert_eq!(
            manager
                .generate_publish_to_client_pkid("client-1", &QoS::AtMostOnce)
                .await,
            1
        );

        let first = manager
            .generate_publish_to_client_pkid("client-1", &QoS::AtLeastOnce)
            .await;
        let second = manager
            .generate_publish_to_client_pkid("client-1", &QoS::ExactlyOnce)
            .await;
        assert_ne!(first, second);
        assert_ne!(first, 0);
        assert_ne!(second, 0);

        manager
            .publish_to_client_pkid_generate
            .store(65535, Ordering::SeqCst);
        let wrapped = manager
            .generate_publish_to_client_pkid("client-2", &QoS::AtLeastOnce)
            .await;
        assert_eq!(wrapped, 1);
    }

    #[test]
    fn publish_ack_data_is_removed_without_affecting_similar_client_ids() {
        let manager = PkidManager::new();
        manager.add_publish_to_client_qos_ack_data("client-1", 2, ack_data(10));
        manager.add_publish_to_client_qos_ack_data("client-10", 2, ack_data(11));
        manager
            .publish_to_client_pkid_cache
            .entry("client-1".to_string())
            .or_default()
            .insert("2".to_string(), 10);

        assert!(manager
            .get_publish_to_client_qos_ack_data("client-1", 2)
            .is_some());
        manager.remove_publish_to_client_pkid("client-1", 2);
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-1", 2)
            .is_none());
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-10", 2)
            .is_some());
        assert!(!manager
            .publish_to_client_pkid_cache
            .contains_key("client-1"));
    }

    #[test]
    fn expired_packet_data_is_pruned_from_all_stores() {
        let manager = PkidManager::new();
        let now_sec = now_second();
        let now_ms = now_millis();

        manager.add_qos_pkid_data("client-1", qos_data(1, now_sec.saturating_sub(61)));
        manager.add_qos_pkid_data("client-1", qos_data(2, now_sec));
        manager
            .publish_to_client_pkid_cache
            .entry("client-1".to_string())
            .or_default()
            .insert("1".to_string(), now_sec.saturating_sub(61));
        manager
            .publish_to_client_pkid_cache
            .entry("client-1".to_string())
            .or_default()
            .insert("2".to_string(), now_sec);
        manager.add_publish_to_client_qos_ack_data(
            "client-1",
            1,
            ack_data(now_ms.saturating_sub(61_000)),
        );
        manager.add_publish_to_client_qos_ack_data("client-1", 2, ack_data(now_ms));

        manager.clean_expired_pkid_data(60);

        assert!(manager.get_qos_pkid_data("client-1", 1).is_none());
        assert!(manager.get_qos_pkid_data("client-1", 2).is_some());
        let cache = manager
            .publish_to_client_pkid_cache
            .get("client-1")
            .unwrap();
        assert!(!cache.contains_key("1"));
        assert!(cache.contains_key("2"));
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-1", 1)
            .is_none());
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-1", 2)
            .is_some());
    }

    #[test]
    fn remove_by_client_id_clears_all_packet_state_for_that_client_only() {
        let manager = PkidManager::new();
        manager.add_qos_pkid_data("client-1", qos_data(1, 10));
        manager.add_publish_to_client_qos_ack_data("client-1", 1, ack_data(10));
        manager
            .publish_to_client_pkid_cache
            .entry("client-1".to_string())
            .or_default()
            .insert("1".to_string(), 10);
        manager.add_qos_pkid_data("client-10", qos_data(1, 10));
        manager.add_publish_to_client_qos_ack_data("client-10", 1, ack_data(10));

        manager.remove_by_client_id("client-1");

        assert_eq!(manager.get_qos_pkid_data_len_by_client_id("client-1"), 0);
        assert!(!manager
            .publish_to_client_pkid_cache
            .contains_key("client-1"));
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-1", 1)
            .is_none());
        assert_eq!(manager.get_qos_pkid_data_len_by_client_id("client-10"), 1);
        assert!(manager
            .get_publish_to_client_qos_ack_data("client-10", 1)
            .is_some());
    }
}
