use self::tcp::{packet::ResponsePackage, tcp_server::TcpServer};
use crate::{
    metadata::{cache::MetadataCache, hearbeat::HeartbeatManager},
    packet::command::Command,
    subscribe::subscribe_manager::SubScribeManager,
};
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info,
};
use flume::{Receiver, Sender};
use protocol::{mqttv4::codec::Mqtt4Codec, mqttv5::codec::Mqtt5Codec};
use std::sync::{Arc, RwLock};

pub mod grpc;
pub mod http;
pub mod quic;
pub mod tcp;
pub mod websocket;

#[derive(Clone, Default, PartialEq, Debug)]
pub enum MQTTProtocol {
    #[default]
    MQTT4,
    MQTT5,
}

impl From<MQTTProtocol> for String {
    fn from(protocol: MQTTProtocol) -> Self {
        match protocol {
            MQTTProtocol::MQTT4 => "MQTT4".into(),
            MQTTProtocol::MQTT5 => "MQTT5".into(),
        }
    }
}

pub async fn start_mqtt_server(
    cache: Arc<RwLock<MetadataCache>>,
    heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_rx4: Receiver<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    response_queue_rx5: Receiver<ResponsePackage>,
) {
    let conf = broker_mqtt_conf();
    if conf.mqtt.mqtt4_enable {
        let command = Command::new(
            MQTTProtocol::MQTT4,
            cache.clone(),
            heartbeat_manager.clone(),
            subscribe_manager.clone(),
        );
        start_mqtt4_server(
            conf,
            command.clone(),
            response_queue_sx4,
            response_queue_rx4,
        )
        .await;
    }

    if conf.mqtt.mqtt5_enable {
        let command = Command::new(
            MQTTProtocol::MQTT5,
            cache.clone(),
            heartbeat_manager.clone(),
            subscribe_manager.clone(),
        );
        start_mqtt5_server(
            conf,
            command.clone(),
            response_queue_sx5,
            response_queue_rx5,
        )
        .await;
    }
}

async fn start_mqtt4_server(
    conf: &BrokerMQTTConfig,
    command: Command,
    response_queue_sx: Sender<ResponsePackage>,
    response_queue_rx: Receiver<ResponsePackage>,
) {
    let port = conf.mqtt.mqtt4_port;
    let codec = Mqtt4Codec::new();
    let server = TcpServer::<Mqtt4Codec>::new(
        MQTTProtocol::MQTT4,
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.request_queue_size,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.response_queue_size,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
        response_queue_sx,
        response_queue_rx,
    );
    server.start(port).await;
    info(format!(
        "MQTT4 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_mqtt5_server(
    conf: &BrokerMQTTConfig,
    command: Command,
    response_queue_sx: Sender<ResponsePackage>,
    response_queue_rx: Receiver<ResponsePackage>,
) {
    let codec = Mqtt5Codec::new();
    let port = conf.mqtt.mqtt5_port;
    let server = TcpServer::<Mqtt5Codec>::new(
        MQTTProtocol::MQTT5,
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.request_queue_size,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.response_queue_size,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
        response_queue_sx,
        response_queue_rx,
    );
    server.start(port).await;
    info(format!(
        "MQTT5 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_quic_server() {}
