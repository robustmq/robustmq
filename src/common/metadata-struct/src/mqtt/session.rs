use common_base::tools::now_second;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MQTTSession {
    pub client_id: String,
    pub session_expiry: u64,
    pub is_contain_last_will: bool,
    pub last_will_delay_interval: Option<u64>,
    pub create_time: u64,

    pub connection_id: Option<u64>,
    pub broker_id: Option<u64>,
    pub reconnect_time: Option<u64>,
    pub distinct_time: Option<u64>,
}

impl MQTTSession {
    pub fn new(
        client_id: String,
        session_expiry: u64,
        is_contain_last_will: bool,
        last_will_delay_interval: Option<u64>,
    ) -> MQTTSession {
        let mut session = MQTTSession::default();
        session.client_id = client_id.clone();
        session.session_expiry = session_expiry;
        session.is_contain_last_will = is_contain_last_will;
        session.last_will_delay_interval = last_will_delay_interval;
        session.create_time = now_second();
        return session;
    }

    pub fn update_connnction_id(&mut self, connection_id: Option<u64>) {
        self.connection_id = connection_id;
    }

    pub fn update_broker_id(&mut self, broker_id: Option<u64>) {
        self.broker_id = broker_id;
    }

    pub fn update_update_time(&mut self) {
        self.reconnect_time = Some(now_second());
    }

    pub fn update_reconnect_time(&mut self) {
        self.reconnect_time = Some(now_second());
    }

    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}
