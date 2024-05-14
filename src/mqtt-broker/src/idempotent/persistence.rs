use axum::async_trait;
use dashmap::DashMap;

use super::{IdempotentData, PacketIdentifierManager};

pub struct IdempotentPersistence {}

impl IdempotentPersistence {
    pub fn new() -> Self {
        return IdempotentPersistence {};
    }
}

#[async_trait]
impl PacketIdentifierManager for IdempotentPersistence {
    async fn save_qos_pkid_data(&self, client_id: String, pkid: u16) {}

    async fn delete_qos_pkid_data(&self, client_id: String, pkid: u16) {}

    async fn get_qos_pkid_data(&self, client_id: String, pkid: u16) -> Option<IdempotentData> {
        return None;
    }

    async fn save_sub_pkid_data(&self, client_id: String, pkid: u16) {}
    async fn delete_sub_pkid_data(&self, client_id: String, pkid: u16) {}
    async fn get_sub_pkid_data(&self, client_id: String, pkid: u16) -> Option<u64> {
        return None;
    }
}
