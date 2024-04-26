use axum::async_trait;
use dashmap::DashMap;

use super::{Idempotent, IdempotentData};

pub struct IdempotentPersistence {}

impl IdempotentPersistence {
    pub fn new() -> Self {
        return IdempotentPersistence {};
    }
}

#[async_trait]
impl Idempotent for IdempotentPersistence {
    async fn save_idem_data(&self, connect_id: u64, pkid: u16) {}

    async fn delete_idem_data(&self, connect_id: u64, pkid: u16) {}

    async fn get_idem_data(&self, connect_id: u64, pkid: u16) -> Option<IdempotentData> {
        return None;
    }

    async fn idem_data(&self) -> DashMap<String, IdempotentData> {
        return DashMap::with_capacity(1);
    }
}
