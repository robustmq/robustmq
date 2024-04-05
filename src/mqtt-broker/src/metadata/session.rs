use std::collections::HashMap;

use protocol::mqtt::{Connect, ConnectProperties, LastWill, LastWillProperties};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Session {
    pub client_id: String,
    pub keep_alive: u16,
    pub last_will: bool,
    pub clean_session: bool,
    pub session_expiry_interval: Option<u32>,
    pub receive_maximum: Option<u16>,
    pub max_packet_size: Option<u32>,
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    pub user_properties: Vec<(String, String)>,
    pub topic_alias: HashMap<u16, String>,
}

impl Session {
    pub fn build_session(
        &self,
        client_id: String,
        connnect: Connect,
        connect_properties: Option<ConnectProperties>,
    ) -> Session {
        let mut session = Session::default();
        session.client_id = client_id;
        session.keep_alive = connnect.keep_alive;
        session.clean_session = connnect.clean_session;

        if let Some(properties) = connect_properties {
            session.session_expiry_interval = properties.session_expiry_interval;
            session.receive_maximum = properties.receive_maximum;
            session.max_packet_size = properties.max_packet_size;
            session.max_packet_size = properties.max_packet_size;
            session.topic_alias_max = properties.topic_alias_max;
            session.request_response_info = properties.request_response_info;
            session.request_problem_info = properties.request_problem_info;
            session.user_properties = properties.user_properties;
        }

        return session;
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct LastWillData {
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
}
