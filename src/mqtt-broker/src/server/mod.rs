use self::tcp::{
    packet::{RequestPackage, ResponsePackage},
    tcp_server::TcpServer,
};
use crate::core::metadata_cache::MetadataCacheManager;
use crate::{
    core::client_heartbeat::HeartbeatManager, handler::command::Command,
    idempotent::memory::IdempotentMemory,
};
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info,
};
use protocol::{mqttv4::codec::Mqtt4Codec, mqttv5::codec::Mqtt5Codec};
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

pub async fn start_mqtt_server<T, S>(
    client_poll: Arc<ClientPool>,
    cache: Arc<MetadataCacheManager>,
    heartbeat_manager: Arc<HeartbeatManager>,
    metadata_storage_adapter: Arc<T>,
    message_storage_adapter: Arc<S>,
    idempotent_manager: Arc<IdempotentMemory>,
    request_queue_sx4: Sender<RequestPackage>,
    request_queue_sx5: Sender<RequestPackage>,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    if conf.mqtt.mqtt4_enable {
        let command = Command::new(
            MQTTProtocol::MQTT4,
            cache.clone(),
            heartbeat_manager.clone(),
            metadata_storage_adapter.clone(),
            message_storage_adapter.clone(),
            response_queue_sx4.clone(),
            idempotent_manager.clone(),
            client_poll.clone(),
        );
        start_mqtt4_server(conf, command.clone(), request_queue_sx4, response_queue_sx4).await;
    }

    if conf.mqtt.mqtt5_enable {
        let command = Command::new(
            MQTTProtocol::MQTT5,
            cache.clone(),
            heartbeat_manager.clone(),
            metadata_storage_adapter.clone(),
            message_storage_adapter.clone(),
            response_queue_sx5.clone(),
            idempotent_manager.clone(),
            client_poll.clone(),
        );
        start_mqtt5_server(conf, command.clone(), request_queue_sx5, response_queue_sx5).await;
    }
}

async fn start_mqtt4_server<T, U>(
    conf: &BrokerMQTTConfig,
    command: Command<T, U>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    U: StorageAdapter + Sync + Send + 'static + Clone,
{
    let port = conf.mqtt.mqtt4_port;
    let codec = Mqtt4Codec::new();
    let server = TcpServer::<Mqtt4Codec, T, U>::new(
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

async fn start_mqtt5_server<T, U>(
    conf: &BrokerMQTTConfig,
    command: Command<T, U>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    U: StorageAdapter + Sync + Send + 'static + Clone,
{
    let codec = Mqtt5Codec::new();
    let port = conf.mqtt.mqtt5_port;
    let server = TcpServer::<Mqtt5Codec, T, U>::new(
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
