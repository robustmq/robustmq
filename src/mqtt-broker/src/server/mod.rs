use self::tcp::server::TcpServer;
use crate::{
    handler::{cache_manager::CacheManager, command::Command},
    subscribe::subscribe_cache::SubscribeCacheManager,
};
use clients::poll::ClientPool;
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info};
use connection_manager::ConnectionManager;
use packet::{RequestPackage, ResponsePackage};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::{
    broadcast,
    mpsc::{self, Sender},
};

pub mod connection;
pub mod connection_manager;
pub mod grpc;
pub mod http;
pub mod packet;
pub mod quic;
pub mod tcp;
pub mod websocket;

pub async fn start_tcp_server<S>(
    sucscribe_manager: Arc<SubscribeCacheManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    let command = Command::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        sucscribe_manager.clone(),
        client_poll.clone(),
        connection_manager.clone(),
        stop_sx.clone(),
    );

    let server = TcpServer::<S>::new(
        command,
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_thread_num,
        stop_sx.clone(),
        connection_manager,
    );

    server.start(conf.mqtt.tcp_port).await;

    // server.start_tls(conf.mqtt.tcps_port).await;
    // info(format!(
    //     "MQTT TCP Server started successfully, listening port: {}",
    //     conf.mqtt.tcp_port
    // ));
}

pub async fn start_tcp_ssl_server() {}

async fn start_quic_server() {}
