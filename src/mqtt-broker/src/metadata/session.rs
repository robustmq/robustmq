use common_base::tools::now_second;
use protocol::mqtt::{Connect, ConnectProperties, LastWill, LastWillProperties};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Session {
    pub client_id: String,
    pub keep_alive: u16,
    pub contain_last_will: bool,
    pub clean_session: bool,
    pub session_expiry_interval: Option<u32>,
    pub receive_maximum: Option<u16>,
    pub max_packet_size: u32,
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    pub user_properties: Vec<(String, String)>,
    pub topic_alias: HashMap<u16, String>,
    pub create_time: u64,
    pub reconnect_time: Option<u64>,
    pub session_present: bool,
}

impl Session {
    pub fn build_session(
        client_id: String,
        connnect: &Connect,
        connect_properties: &Option<ConnectProperties>,
        server_max_keep_alive: u16,
        contain_last_will: bool,
    ) -> Session {
        let mut session = Session::default();
        session.client_id = client_id.clone();
        session.keep_alive = Session::keep_alive(server_max_keep_alive, connnect.keep_alive);
        session.clean_session = connnect.clean_session;
        session.contain_last_will = contain_last_will;
        session.session_present = false;
        session.create_time = now_second();
        session.reconnect_time = None;
        if let Some(properties) = connect_properties {
            session.session_expiry_interval = properties.session_expiry_interval;
            session.receive_maximum = properties.receive_maximum;
            session.max_packet_size = if let Some(da) = properties.max_packet_size{
                    da
            }else{
                1024*1024
            };
            session.topic_alias_max = properties.topic_alias_max;
            session.request_response_info = properties.request_response_info;
            session.request_problem_info = properties.request_problem_info;
            session.user_properties = properties.user_properties.clone();
        }

        return session;
    }

    pub fn keep_alive(server_keep_alive: u16, client_keep_alive: u16) -> u16 {
        return std::cmp::min(server_keep_alive, client_keep_alive);
    }

    pub fn update_reconnect_time(&mut self) {
        self.reconnect_time = Some(now_second());
        self.session_present = true;
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct LastWillData {
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
}
