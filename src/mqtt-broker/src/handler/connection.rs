use std::sync::Arc;

use clients::poll::ClientPool;
use common_base::{
    errors::RobustMQError, log::info, tools::{now_second, unique_id}
};
use dashmap::DashMap;
use metadata_struct::mqtt::cluster::MQTTCluster;
use protocol::mqtt::common::{Connect, ConnectProperties};
use serde::{Deserialize, Serialize};

use crate::{server::connection_manager::ConnectionManager, storage::session::SessionStorage};

use super::cache_manager::{self, CacheManager};

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
        client_id: &String,
        receive_maximum: u16,
        max_packet_size: u32,
        topic_alias_max: u16,
        request_problem_info: u8,
        keep_alive: u32,
    ) -> Connection {
        let mut conn = Connection::default();
        conn.connect_id = connect_id;
        conn.client_id = client_id.clone();
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

pub fn build_connection(
    connect_id: u64,
    client_id: &String,
    cluster: &MQTTCluster,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
) -> Connection {
    let keep_alive = std::cmp::min(cluster.server_keep_alive(), connect.keep_alive);
    let (receive_maximum, max_packet_size, topic_alias_max, request_problem_info) =
        if let Some(properties) = connect_properties {
            let receive_maximum = if let Some(value) = properties.receive_maximum {
                std::cmp::min(value, cluster.receive_max())
            } else {
                cluster.receive_max()
            };

            let max_packet_size = if let Some(value) = properties.max_packet_size {
                std::cmp::min(value, cluster.max_packet_size())
            } else {
                cluster.max_packet_size()
            };

            let topic_alias_max = if let Some(value) = properties.topic_alias_max {
                std::cmp::min(value, cluster.topic_alias_max())
            } else {
                cluster.topic_alias_max()
            };

            let request_problem_info = if let Some(value) = properties.request_problem_info {
                value
            } else {
                0
            };

            (
                receive_maximum,
                max_packet_size,
                topic_alias_max,
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
        &client_id,
        receive_maximum,
        max_packet_size,
        topic_alias_max,
        request_problem_info,
        keep_alive as u32,
    );
}

pub fn get_client_id(client_id: &String) -> (String, bool) {
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

pub async fn disconnect_connection(
    client_id: &String,
    connect_id: u64,
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    connnection_manager: &Arc<ConnectionManager>,
) -> Result<(), RobustMQError> {
    cache_manager.remove_connection(connect_id);
    cache_manager.update_session_connect_id(client_id, None);

    let session_storage = SessionStorage::new(client_poll.clone());

    match session_storage
        .update_session(client_id, 0, 0, 0, now_second())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }

    connnection_manager.clonse_connect(connect_id).await;
    return Ok(());
}

#[cfg(test)]
mod test {
    use super::build_connection;
    use super::get_client_id;
    use super::response_information;
    use super::REQUEST_RESPONSE_PREFIX_NAME;
    use metadata_struct::mqtt::cluster::MQTTCluster;
    use protocol::mqtt::common::Connect;
    use protocol::mqtt::common::ConnectProperties;

    #[tokio::test]
    pub async fn build_connection_test() {
        let connect_id = 1;
        let client_id = "client_id-***".to_string();
        let cluster = MQTTCluster::new();
        let connect = Connect {
            keep_alive: 10,
            client_id: client_id.clone(),
            clean_session: true,
        };
        let connect_properties = ConnectProperties {
            session_expiry_interval: Some(60),
            receive_maximum: Some(100),
            max_packet_size: Some(100),
            request_problem_info: Some(0),
            request_response_info: Some(0),
            topic_alias_max: Some(100),
            user_properties: Vec::new(),
            authentication_method: None,
            authentication_data: None,
        };
        let mut conn = build_connection(
            connect_id,
            &client_id,
            &cluster,
            &connect,
            &Some(connect_properties),
        );
        assert_eq!(conn.connect_id, connect_id);
        assert_eq!(conn.client_id, client_id);
        assert!(!conn.login);
        conn.login_success();
        assert!(conn.login);
        assert_eq!(conn.keep_alive, 10);
        assert_eq!(conn.receive_maximum, 100);
        assert_eq!(conn.max_packet_size, 100);
        assert_eq!(conn.topic_alias_max, 100);
        assert_eq!(conn.request_problem_info, 0);
    }

    #[tokio::test]
    pub async fn get_client_id_test() {
        let client_id = "".to_string();
        let (new_client_id, is_new) = get_client_id(&client_id);
        assert!(is_new);
        assert!(!new_client_id.is_empty());

        let client_id = "client_id-***".to_string();
        let (new_client_id, is_new) = get_client_id(&client_id);
        assert!(!is_new);
        assert_eq!(new_client_id, client_id);
        assert!(!new_client_id.is_empty());
    }

    #[tokio::test]
    pub async fn response_information_test() {
        let mut connect_properties = ConnectProperties::default();
        connect_properties.request_response_info = Some(1);
        let res = response_information(&Some(connect_properties));
        assert_eq!(res.unwrap(), REQUEST_RESPONSE_PREFIX_NAME.to_string());

        let res = response_information(&Some(ConnectProperties::default()));
        assert!(res.is_none());

        let mut connect_properties = ConnectProperties::default();
        connect_properties.request_response_info = Some(0);
        let res = response_information(&Some(connect_properties));
        assert!(res.is_none());
    }
}
