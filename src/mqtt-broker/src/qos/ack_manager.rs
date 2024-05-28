use dashmap::DashMap;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct AckPacketInfo {
    pub sx: Sender<AckPackageData>,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct AckPackageData {
    pub ack_type: AckPackageType,
    pub pkid: u16,
}

#[derive(Clone, PartialEq, PartialOrd,Debug)]
pub enum AckPackageType {
    PubAck,
    PubComp,
    PubRel,
    PubRec,
}

#[derive(Clone)]
pub struct AckManager {
    //(client_id_pkid, send)
    qos_ack_pkid_info: DashMap<String, AckPacketInfo>,
}

impl AckManager {
    pub fn new() -> Self {
        return AckManager {
            qos_ack_pkid_info: DashMap::with_capacity(8),
        };
    }

    pub fn add(&self, client_id: String, pkid: u16, packet: AckPacketInfo) {
        let key = self.keys(client_id, pkid);
        self.qos_ack_pkid_info.insert(key, packet);
    }

    pub fn remove(&self, client_id: String, pkid: u16) {
        let key = self.keys(client_id, pkid);
        self.qos_ack_pkid_info.remove(&key);
    }

    pub fn get(&self, client_id: String, pkid: u16) -> Option<AckPacketInfo> {
        let key = self.keys(client_id, pkid);
        if let Some(data) = self.qos_ack_pkid_info.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    fn keys(&self, client_id: String, pkid: u16) -> String {
        return format!("{}_{}", client_id, pkid);
    }
}
