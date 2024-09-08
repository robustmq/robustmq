use clients::poll::ClientPool;
use common_base::tools::{get_local_ip, now_mills};
use log::error;
use metadata_struct::mqtt::{message::MQTTMessage, session::MQTTSession};
use protocol::mqtt::common::MQTTProtocol;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::storage::StorageAdapter;

use crate::{
    handler::{cache::CacheManager, connection::Connection},
    server::connection_manager::ConnectionManager,
};

use super::{write_topic_data, SYSTEM_TOPIC_BROKERS_CONNECTED};

#[derive(Default, Serialize, Deserialize)]
pub struct SystemTopicConnectedEventMessge {
    pub username: String,
    pub ts: u128,
    pub sockport: u16,
    pub proto_ver: Option<MQTTProtocol>,
    pub proto_name: String,
    pub keepalive: u16,
    pub ipaddress: String,
    pub expiry_interval: u64,
    pub connected_at: u128,
    pub connack: u16,
    pub clientid: String,
    pub clean_start: bool,
}

#[derive(Default)]
pub struct SystemTopicDisConnectedEventMessge {
    pub username: String,
    pub ts: u128,
    pub sockport: u32,
    pub reason: String,
    pub proto_ver: u32,
    pub proto_name: String,
    pub ipaddress: String,
    pub disconnected_at: u128,
    pub clientid: String,
}
#[derive(Default)]
pub struct SystemTopicSubscribedEventMessge {
    pub username: String,
    pub ts: u128,
    pub subopts: SystemTopicSubscribedEventMessgeSUbopts,
    pub topic: String,
    pub protocol: String,
    pub clientid: String,
}

#[derive(Default)]
pub struct SystemTopicSubscribedEventMessgeSUbopts {
    pub sub_props: HashMap<String, String>,
    pub rh: u16,
    pub rap: u16,
    pub qos: u16,
    pub nl: u16,
    pub is_new: bool,
}

#[derive(Default)]
pub struct SystemTopicUnSubscribedEventMessge {
    pub username: String,
    pub ts: u128,
    pub topic: String,
    pub protocol: String,
    pub clientid: String,
}

pub async fn st_report_connected_event<S>(
    message_storage_adapter: &Arc<S>,
    metadata_cache: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    session: &MQTTSession,
    connection: &Connection,
    connect_id: u64,
    connnection_manager: &Arc<ConnectionManager>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    if let Some(network_connection) = connnection_manager.get_connect(connect_id) {
        let event_data = SystemTopicConnectedEventMessge {
            username: connection.login_user.clone(),
            ts: now_mills(),
            sockport: network_connection.addr.port(),
            proto_ver: network_connection.protocol.clone(),
            proto_name: "MQTT".to_string(),
            keepalive: connection.keep_alive,
            ipaddress: connection.source_ip_addr.clone(),
            expiry_interval: session.session_expiry,
            connected_at: now_mills(),
            connack: 1,
            clientid: session.client_id.to_string(),
            clean_start: false,
        };
        match serde_json::to_string(&event_data) {
            Ok(data) => {
                let topic_name = replace_name(
                    SYSTEM_TOPIC_BROKERS_CONNECTED.to_string(),
                    session.client_id.to_string(),
                );

                if let Some(record) =
                    MQTTMessage::build_system_topic_message(topic_name.clone(), data)
                {
                    write_topic_data(
                        message_storage_adapter,
                        metadata_cache,
                        client_poll,
                        topic_name,
                        record,
                    )
                    .await;
                }
            }
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
    }
}

pub fn st_report_disconnected_event() {}

pub fn st_report_subscribed_event() {}

pub fn st_report_unsubscribed_event() {}

fn replace_name(mut topic_name: String, client_id: String) -> String {
    if topic_name.contains("${node}") {
        let local_ip = get_local_ip();
        topic_name = topic_name.replace("${node}", &local_ip)
    }
    if topic_name.contains("${clientid}") {
        topic_name = topic_name.replace("${clientid}", &client_id)
    }
    return topic_name;
}
