use axum::async_trait;

pub mod memory;
pub mod persistence;

#[async_trait]
pub trait Idempotent {
    async fn save_idem_data(&self, topic_id: String, pkid: u16);
    async fn delete_idem_data(&self, topic_id: String, pkid: u16);
    async fn idem_data_exists(&self, topic_id: String, pkid: u16) -> bool;
}
