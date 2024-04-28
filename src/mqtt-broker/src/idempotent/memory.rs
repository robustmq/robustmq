use super::{Idempotent, IdempotentData};
use axum::async_trait;
use common_base::tools::now_second;
use dashmap::DashMap;

#[derive(Clone)]
pub struct IdempotentMemory {
    //
    pkid_data: DashMap<String, IdempotentData>,
    client_id_pkid: DashMap<String, DashMap<u16, u64>>,
}

impl IdempotentMemory {
    pub fn new() -> Self {
        let pkid_data = DashMap::with_capacity(256);
        return IdempotentMemory {
            pkid_data,
            client_id_pkid: DashMap::with_capacity(256),
        };
    }

    fn iden_key(&self, client_id: String, pkid: u16) -> String {
        return format!("{}_{}", client_id, pkid);
    }
}

#[async_trait]
impl Idempotent for IdempotentMemory {
    async fn save_idem_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id.clone(), pkid);
        self.pkid_data.insert(
            key,
            IdempotentData {
                client_id,
                create_time: now_second(),
            },
        );
    }

    async fn delete_idem_data(&self, client_id: String, pkid: u16) {
        let key = self.iden_key(client_id, pkid);
        self.pkid_data.remove(&key);
    }

    async fn get_idem_data(&self, client_id: String, pkid: u16) -> Option<IdempotentData> {
        let key = self.iden_key(client_id, pkid);
        if let Some(data) = self.pkid_data.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    async fn idem_data(&self) -> DashMap<String, IdempotentData> {
        return self.pkid_data.clone();
    }
}
