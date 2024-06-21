use self::tcp::{
    packet::{RequestPackage, ResponsePackage},
    tcp_server::TcpServer,
};
use crate::{
    core::qos_manager::QosManager, core::heartbeat_cache::HeartbeatCache,
    handler::command::Command,
};
use crate::{
    core::metadata_cache::MetadataCacheManager, subscribe::subscribe_cache::SubscribeCache,
};
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info,
};
use protocol::mqtt::{mqttv4::codec::Mqtt4Codec, mqttv5::codec::Mqtt5Codec};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

pub mod grpc;
pub mod http;
pub mod quic;
pub mod tcp;
pub mod websocket;

#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
pub enum MQTTProtocol {
    #[default]
    MQTT3,
    MQTT4,
    MQTT5,
}

impl From<MQTTProtocol> for String {
    fn from(protocol: MQTTProtocol) -> Self {
        match protocol {
            MQTTProtocol::MQTT3 => "MQTT3".into(),
            MQTTProtocol::MQTT4 => "MQTT4".into(),
            MQTTProtocol::MQTT5 => "MQTT5".into(),
        }
    }
}

pub async fn start_mqtt_server<S>(
    sucscribe_manager: Arc<SubscribeCache>,
    cache: Arc<MetadataCacheManager>,
    heartbeat_manager: Arc<HeartbeatCache>,
    message_storage_adapter: Arc<S>,
    qos_manager: Arc<QosManager>,
    client_poll: Arc<ClientPool>,
    request_queue_sx4: Sender<RequestPackage>,
    request_queue_sx5: Sender<RequestPackage>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    if conf.mqtt.mqtt4_enable {
        let command = Command::new(
            MQTTProtocol::MQTT4,
            cache.clone(),
            heartbeat_manager.clone(),
            message_storage_adapter.clone(),
            response_queue_sx4.clone(),
            qos_manager.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
        );
        start_mqtt4_server(conf, command.clone(), request_queue_sx4, response_queue_sx4).await;
    }

    if conf.mqtt.mqtt5_enable {
        let command = Command::new(
            MQTTProtocol::MQTT5,
            cache.clone(),
            heartbeat_manager.clone(),
            message_storage_adapter.clone(),
            response_queue_sx5.clone(),
            qos_manager.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
        );
        start_mqtt5_server(conf, command.clone(), request_queue_sx5, response_queue_sx5).await;
    }
}

async fn start_mqtt4_server<S>(
    conf: &BrokerMQTTConfig,
    command: Command<S>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let port = conf.mqtt.mqtt4_port;
    let codec = Mqtt4Codec::new();
    let server = TcpServer::<Mqtt4Codec, S>::new(
        MQTTProtocol::MQTT4,
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
        request_queue_sx,
        response_queue_sx,
    );
    server.start(port).await;
    info(format!(
        "MQTT4 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_mqtt5_server<S>(
    conf: &BrokerMQTTConfig,
    command: Command<S>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let codec = Mqtt5Codec::new();
    let port = conf.mqtt.mqtt5_port;
    let server = TcpServer::<Mqtt5Codec, S>::new(
        MQTTProtocol::MQTT5,
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
        request_queue_sx,
        response_queue_sx,
    );
    server.start(port).await;
    info(format!(
        "MQTT5 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_quic_server() {}
