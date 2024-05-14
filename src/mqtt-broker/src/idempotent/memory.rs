use super::{IdempotentData, PacketIdentifierManager};
use axum::async_trait;
use common_base::tools::now_second;
use dashmap::DashMap;

#[derive(Clone)]
pub struct PacketIdentifierMemory {
    // (client_id_pkid, IdempotentData)
    qos_pkid_data: DashMap<String, IdempotentData>,
    // (client_id_pkid, time_sec)
    sub_pkid_data: DashMap<String, u64>,
}

impl PacketIdentifierMemory {
    pub fn new() -> Self {
        return PacketIdentifierMemory {
            qos_pkid_data: DashMap::with_capacity(256),
            sub_pkid_data: DashMap::with_capacity(256),
        };
    }

    fn iden_key(&self, client_id: String, pkid: u16) -> String {
        return format!("{}_{}", client_id, pkid);
    }
}

#[async_trait]
impl PacketIdentifierManager for PacketIdentifierMemory {
    async fn save_qos_pkid_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id.clone(), pkid);
        self.qos_pkid_data.insert(
            key,
            IdempotentData {
                client_id,
                create_time: now_second(),
            },
        );
    }

    async fn delete_qos_pkid_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id, pkid);
        self.qos_pkid_data.remove(&key);
    }

    async fn get_qos_pkid_data(&self, client_id: String, pkid: u16) -> Option<IdempotentData> {
        let key = self.iden_key(client_id, pkid);
        if let Some(data) = self.qos_pkid_data.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    async fn save_sub_pkid_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id.clone(), pkid);
        self.sub_pkid_data.insert(key, now_second());
    }
    async fn delete_sub_pkid_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id, pkid);
        self.sub_pkid_data.remove(&key);
    }

    async fn get_sub_pkid_data(&self, client_id: String, pkid: u16) -> Option<u64> {
        let key = self.iden_key(client_id, pkid);
        if let Some(data) = self.sub_pkid_data.get(&key) {
            return Some(data.clone());
        }
        return None;
    }
}
