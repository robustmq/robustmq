use axum::async_trait;
use common_base::log::info;
use dashmap::DashMap;

pub mod memory;
pub mod persistence;

#[async_trait]
pub trait Idempotent {
    async fn save_idem_data(&self, topic_id: String, pkid: u16);
    async fn delete_idem_data(&self, topic_id: String, pkid: u16);
    async fn idem_data_exists(&self, topic_id: String, pkid: u16) -> bool;
    async fn idem_data(&self) -> DashMap<String,u64>;
}

pub struct IdempotentCleanThread {}

impl IdempotentCleanThread {
    pub fn new() -> IdempotentCleanThread {
        return IdempotentCleanThread {};
    }

    pub fn start(&self) {
        info("Idempotent message data cleaning thread started successfully".to_string());
    }
}
