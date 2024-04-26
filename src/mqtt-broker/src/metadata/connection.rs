use common_base::tools::now_second;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Connection {
    pub connect_id: u64,
    pub client_id: String,
    pub login: bool,
    pub create_time: u64,
    pub topic_alias: DashMap<u16, String>,
}

impl Connection {
    pub fn new(connect_id: u64, client_id: String) -> Connection {
        return Connection {
            connect_id,
            client_id,
            login: false,
            create_time: now_second(),
            topic_alias: DashMap::with_capacity(64),
        };
    }
}
