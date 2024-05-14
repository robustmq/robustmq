use axum::async_trait;
use common_base::log::info;

pub mod memory;
pub mod persistence;

#[derive(Clone)]
pub struct IdempotentData {
    pub client_id: String,
    pub create_time: u64,
}

#[async_trait]
pub trait PacketIdentifierManager {
    // qos
    async fn save_qos_pkid_data(&self, client_id: String, pkid: u16);
    async fn delete_qos_pkid_data(&self, client_id: String, pkid: u16);
    async fn get_qos_pkid_data(&self, client_id: String, pkid: u16) -> Option<IdempotentData>;

    // sub
    async fn save_sub_pkid_data(&self, client_id: String, pkid: u16);
    async fn delete_sub_pkid_data(&self, client_id: String, pkid: u16);
    async fn get_sub_pkid_data(&self, client_id: String, pkid: u16) -> Option<u64>;
}

pub struct IdempotentCleanManager {}

impl IdempotentCleanManager {
    pub fn new() -> IdempotentCleanManager {
        return IdempotentCleanManager {};
    }

    pub fn start(&self) {
        info("Idempotent message data cleaning thread started successfully".to_string());
    }
}
