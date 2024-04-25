use super::Idempotent;
use axum::async_trait;
use common_base::tools::now_second;
use dashmap::DashMap;

pub struct IdempotentMemory {
    pkid_data: DashMap<String, u64>,
}

impl IdempotentMemory {
    pub fn new() -> Self {
        let pkid_data = DashMap::with_capacity(256);
        return IdempotentMemory { pkid_data };
    }

    fn iden_key(&self, topic_id: String, pkid: u16) -> String {
        return format!("{}_{}", topic_id, pkid);
    }
}

#[async_trait]
impl Idempotent for IdempotentMemory {
    async fn save_idem_data(&self, topic_id: String, pkid: u16) {
        let key = self.iden_key(topic_id, pkid);
        self.pkid_data.insert(key, now_second());
    }

    async fn delete_idem_data(&self, topic_id: String, pkid: u16) {
        let key = self.iden_key(topic_id, pkid);
        self.pkid_data.remove(&key);
    }

    async fn idem_data_exists(&self, topic_id: String, pkid: u16) -> bool {
        let key = self.iden_key(topic_id, pkid);
        return self.pkid_data.contains_key(&key);
    }
}
