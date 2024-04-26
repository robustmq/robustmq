use axum::async_trait;
use common_base::log::info;
use dashmap::DashMap;

pub mod memory;
pub mod persistence;

#[derive(Clone)]
pub struct IdempotentData {
    pub connect_id: u64,
    pub create_time: u64,
}

#[async_trait]
pub trait Idempotent {
    async fn save_idem_data(&self, connect_id: u64, pkid: u16);
    async fn delete_idem_data(&self, connect_id: u64, pkid: u16);
    async fn get_idem_data(&self, connect_id: u64, pkid: u16) -> Option<IdempotentData>;
    async fn idem_data(&self) -> DashMap<String, IdempotentData>;
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
