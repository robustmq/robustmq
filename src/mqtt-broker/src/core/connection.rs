use common_base::tools::{now_second, unique_id};
use dashmap::DashMap;
use metadata_struct::mqtt::cluster::MQTTCluster;
use protocol::mqtt::common::{Connect, ConnectProperties};
use serde::{Deserialize, Serialize};

pub const REQUEST_RESPONSE_PREFIX_NAME: &str = "/sys/request_response/";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Connection {
    pub connect_id: u64,
    pub client_id: String,
    pub login: bool,
    pub keep_alive: u32,
    pub topic_alias: DashMap<u16, String>,
    pub receive_maximum: u16,
    pub max_packet_size: u32,
    pub topic_alias_max: u16,
    pub request_problem_info: u8,
    pub create_time: u64,
}

impl Connection {
    pub fn new(
        connect_id: u64,
        client_id: String,
        receive_maximum: u16,
        max_packet_size: u32,
        topic_alias_max: u16,
        request_problem_info: u8,
        keep_alive: u32,
    ) -> Connection {
        let mut conn = Connection::default();
        conn.connect_id = connect_id;
        conn.client_id = client_id;
        conn.login = false;
        conn.keep_alive = keep_alive;
        conn.receive_maximum = receive_maximum;
        conn.max_packet_size = max_packet_size;
        conn.topic_alias_max = topic_alias_max;
        conn.request_problem_info = request_problem_info;
        conn.create_time = now_second();
        return conn;
    }

    pub fn login_success(&mut self) {
        self.login = true;
    }

    pub fn is_response_proplem_info(&self) -> bool {
        return self.request_problem_info == 1;
    }
}

pub fn create_connection(
    connect_id: u64,
    client_id: String,
    cluster: &MQTTCluster,
    connect: Connect,
    connect_properties: Option<ConnectProperties>,
) -> Connection {
    let keep_alive = client_keep_alive(cluster.server_keep_alive(), connect.keep_alive);
    let (receive_maximum, max_packet_size, topic_alias_max, request_problem_info) =
        if let Some(properties) = connect_properties {
            let receive_maximum = if let Some(value) = properties.receive_maximum {
                value
            } else {
                u16::MAX
            };

            let max_packet_size = if let Some(value) = properties.max_packet_size {
                value
            } else {
                u32::MAX
            };

            let topic_alias_max = if let Some(value) = properties.topic_alias_max {
                value
            } else {
                u16::MAX
            };

            let request_problem_info = if let Some(value) = properties.request_problem_info {
                value
            } else {
                0
            };

            (
                std::cmp::min(receive_maximum, cluster.receive_max()),
                std::cmp::min(max_packet_size, cluster.max_packet_size()),
                std::cmp::min(topic_alias_max, cluster.topic_alias_max()),
                request_problem_info,
            )
        } else {
            (
                cluster.receive_max(),
                cluster.max_packet_size(),
                cluster.topic_alias_max(),
                0,
            )
        };
    return Connection::new(
        connect_id,
        client_id,
        receive_maximum,
        max_packet_size,
        topic_alias_max,
        request_problem_info,
        keep_alive as u32,
    );
}

pub fn client_keep_alive(server_keep_alive: u16, client_keep_alive: u16) -> u16 {
    return std::cmp::min(server_keep_alive, client_keep_alive);
}

pub fn get_client_id(client_id: String) -> (String, bool) {
    let (client_id, new_client_id) = if client_id.is_empty() {
        (unique_id(), true)
    } else {
        (client_id.clone(), false)
    };

    return (client_id, new_client_id);
}

pub fn response_information(connect_properties: &Option<ConnectProperties>) -> Option<String> {
    if let Some(properties) = connect_properties {
        if let Some(request_response_info) = properties.request_response_info {
            if request_response_info == 1 {
                return Some(REQUEST_RESPONSE_PREFIX_NAME.to_string());
            }
        }
    }
    return None;
}
