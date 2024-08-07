// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use super::connection_manager::ConnectionManager;
use crate::{
    handler::{cache_manager::CacheManager, command::Command}, security::AuthDriver, subscribe::subscribe_manager::SubscribeManager
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
    sucscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
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
        auth_driver.clone(),
    );

    let server = TcpServer::<S>::new(
        command.clone(),
        conf.tcp_thread.accept_thread_num,
        conf.tcp_thread.handler_thread_num,
        conf.tcp_thread.response_thread_num,
        stop_sx.clone(),
        connection_manager.clone(),
        sucscribe_manager.clone(),
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
        sucscribe_manager.clone(),
        cache_manager,
        client_poll,
    );
    server.start_tls(conf.network.tcps_port).await;
}
