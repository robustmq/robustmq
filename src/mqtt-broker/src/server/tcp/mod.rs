use super::connection_manager::ConnectionManager;
use crate::{
    handler::{cache_manager::CacheManager, command::Command},
    subscribe::subscribe_cache::SubscribeCacheManager,
};
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use server::TcpServer;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast;

pub mod server;
pub mod tls_server;

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
    );

    let server = TcpServer::<S>::new(
        command.clone(),
        conf.tcp_thread.accept_thread_num,
        conf.tcp_thread.handler_thread_num,
        conf.tcp_thread.response_thread_num,
        stop_sx.clone(),
        connection_manager.clone(),
        cache_manager.clone(),
        client_poll.clone(),
    );
    server.start(conf.network.tcp_port).await;

    let server = TcpServer::<S>::new(
        command,
        conf.tcp_thread.accept_thread_num,
        conf.tcp_thread.handler_thread_num,
        conf.tcp_thread.response_thread_num,
        stop_sx.clone(),
        connection_manager,
        cache_manager,
        client_poll,
    );
    server.start_tls(conf.network.tcps_port).await;
}
