use self::tcp::{
    packet::{RequestPackage, ResponsePackage},
    server::TcpServer,
};
use crate::handler::command::Command;
use crate::{core::cache_manager::CacheManager, subscribe::subscribe_cache::SubscribeCacheManager};
use clients::poll::ClientPool;
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info};
use protocol::mqtt::common::MQTTProtocol;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

pub mod grpc;
pub mod http;
pub mod quic;
pub mod tcp;
pub mod websocket;

pub async fn start_tcp_server<S>(
    sucscribe_manager: Arc<SubscribeCacheManager>,
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    let command = Command::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        response_queue_sx.clone(),
        sucscribe_manager.clone(),
        client_poll.clone(),
    );

    let server = TcpServer::<S>::new(
        MQTTProtocol::MQTT5,
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        request_queue_sx,
        response_queue_sx,
    );

    server.start(conf.mqtt.tcp_port).await;
    info(format!(
        "MQTT TCP Server started successfully, listening port: {}",
        conf.mqtt.tcp_port
    ));
}

pub async fn start_tcp_ssl_server() {}

pub async fn start_websockets_server() {}

pub async fn start_websockets_ssl_server() {}
async fn start_quic_server() {}
