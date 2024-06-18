use common_base::tools::now_second;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct QosAckPacketInfo {
    pub sx: Sender<QosAckPackageData>,
    pub create_time: u64,
}

#[derive(Clone, Debug)]
pub struct QosAckPackageData {
    pub ack_type: QosAckPackageType,
    pub pkid: u16,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum QosAckPackageType {
    PubAck,
    PubComp,
    PubRel,
    PubRec,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientPkidData {
    pub client_id: String,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct QosManager {
    //(client_id_pkid, AckPacketInfo)
    pub qos_ack_packet: DashMap<String, QosAckPacketInfo>,

    // (client_id_pkid, QosPkidData)
    pub client_pkid_data: DashMap<String, ClientPkidData>,
}

impl QosManager {
    pub fn new() -> Self {
        return QosManager {
            qos_ack_packet: DashMap::with_capacity(8),
            client_pkid_data: DashMap::with_capacity(8),
        };
    }

    pub fn add_ack_packet(&self, client_id: String, pkid: u16, packet: QosAckPacketInfo) {
        let key = self.key(client_id, pkid);
        self.qos_ack_packet.insert(key, packet);
    }

    pub fn remove_ack_packet(&self, client_id: String, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.qos_ack_packet.remove(&key);
    }

    pub fn get_ack_packet(&self, client_id: String, pkid: u16) -> Option<QosAckPacketInfo> {
        let key = self.key(client_id, pkid);
        if let Some(data) = self.qos_ack_packet.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    async fn add_client_pkid(&self, client_id: String, pkid: u16) {
        let key = self.key(client_id.clone(), pkid);
        self.client_pkid_data.insert(
            key,
            ClientPkidData {
                client_id,
                create_time: now_second(),
            },
        );
    }

    async fn delete_client_pkid(&self, client_id: String, pkid: u16) {
        let key = self.key(client_id, pkid);
        self.client_pkid_data.remove(&key);
    }

    async fn get_client_pkid(&self, client_id: String, pkid: u16) -> Option<ClientPkidData> {
        let key = self.key(client_id, pkid);
        if let Some(data) = self.client_pkid_data.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    fn key(&self, client_id: String, pkid: u16) -> String {
        return format!("{}_{}", client_id, pkid);
    }
}
