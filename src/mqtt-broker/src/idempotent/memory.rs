use super::{Idempotent, IdempotentData};
use axum::async_trait;
use common_base::tools::now_second;
use dashmap::DashMap;

#[derive(Clone)]
pub struct IdempotentMemory {
    pkid_data: DashMap<String, IdempotentData>,
}

impl IdempotentMemory {
    pub fn new() -> Self {
        let pkid_data = DashMap::with_capacity(256);
        return IdempotentMemory { pkid_data };
    }

    fn iden_key(&self, connect_id: u64, pkid: u16) -> String {
        return format!("{}_{}", connect_id, pkid);
    }
}

#[async_trait]
impl Idempotent for IdempotentMemory {
    async fn save_idem_data(&self, connect_id: u64, pkid: u16) {
        let key = self.iden_key(connect_id, pkid);
        self.pkid_data.insert(
            key,
            IdempotentData {
                connect_id,
                create_time: now_second(),
            },
        );
    }

    async fn delete_idem_data(&self, connect_id: u64, pkid: u16) {
        let key = self.iden_key(connect_id, pkid);
        self.pkid_data.remove(&key);
    }

    async fn get_idem_data(&self, connect_id: u64, pkid: u16) -> Option<IdempotentData> {
        let key = self.iden_key(connect_id, pkid);
        if let Some(data) = self.pkid_data.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    async fn idem_data(&self) -> DashMap<String, IdempotentData> {
        return self.pkid_data.clone();
    }
}
