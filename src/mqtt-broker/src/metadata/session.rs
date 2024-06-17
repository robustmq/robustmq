use common_base::tools::now_second;
use protocol::mqtt::{LastWill, LastWillProperties};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Session {
    pub client_id: String,
    pub last_will_delay_interval: u32,
    pub last_will: Option<LastWillData>,
    pub session_expiry: u32,
    pub connection_id: Option<u64>,
    pub reconnect_time: Option<u64>,
}

impl Session {
    pub fn new(
        client_id: String,
        session_expiry: u32,
        last_will: Option<LastWillData>,
        delay_interval: u32,
    ) -> Session {
        let mut session = Session::default();
        session.client_id = client_id.clone();
        session.last_will = last_will;
        session.session_expiry = session_expiry;
        session.last_will_delay_interval = delay_interval;
        return session;
    }

    pub fn update_connnction_id(&mut self, connection_id: u64) {
        self.connection_id = Some(connection_id);
    }

    pub fn update_reconnect_time(&mut self) {
        self.reconnect_time = Some(now_second());
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LastWillData {
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
}
