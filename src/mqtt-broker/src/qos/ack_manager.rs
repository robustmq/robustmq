use dashmap::DashMap;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AckPacketInfo {
    pub sx: Sender<bool>,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct AckManager {
    //(client_id_pkid, send)
    qos_ack_pkg_info: DashMap<String, AckPacketInfo>,
}

impl AckManager {
    pub fn new() -> Self {
        return AckManager {
            qos_ack_pkg_info: DashMap::with_capacity(8),
        };
    }

    pub fn add(&self, client_id: String, pkid: u16, packet: AckPacketInfo) {
        let key = self.keys(client_id, pkid);
        self.qos_ack_pkg_info.insert(key, packet);
    }

    pub fn remove(&self, client_id: String, pkid: u16) {
        let key = self.keys(client_id, pkid);
        self.qos_ack_pkg_info.remove(&key);
    }

    pub fn contain(&self, client_id: String, pkid: u16) -> bool {
        let key = self.keys(client_id, pkid);
        return self.qos_ack_pkg_info.contains_key(&key);
    }

    fn keys(&self, client_id: String, pkid: u16) -> String {
        return format!("{}_{}", client_id, pkid);
    }
}
