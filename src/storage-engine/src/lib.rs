use cluster::{register_storage_engine_node, unregister_storage_engine_node};
use common::{
    config::storage_engine::StorageEngineConfig, log::info_meta, runtime::create_runtime,
};
use protocol::storage_engine::storage::storage_engine_service_server::StorageEngineServiceServer;
use services::StorageService;
use std::thread::{self, JoinHandle};
use tokio::{runtime::Runtime, signal, sync::broadcast};
use tonic::transport::Server;

mod cluster;
mod index;
mod raft_group;
mod record;
mod segment;
mod services;
mod shard;
mod storage;
mod v1;
mod v2;

pub struct StorageEngine {
    config: StorageEngineConfig,
    stop_send: broadcast::Sender<bool>,
    server_runtime: Runtime,
    daemon_runtime: Runtime,
}

impl StorageEngine {
    pub fn new(config: StorageEngineConfig, stop_send: broadcast::Sender<bool>) -> Self {
        let server_runtime =
            create_runtime("storage-engine-server-runtime", config.runtime_work_threads);

        let daemon_runtime = create_runtime("daemon-runtime", config.runtime_work_threads);
        return StorageEngine {
            config,
            stop_send,
            server_runtime,
            daemon_runtime,
        };
    }

    pub async fn start(&self) {
        // Register Node
        // register_storage_engine_node(self.config.clone()).await;

        // start GRPC && HTTP Server
        self.start_server().await;

        // Threads that run the daemon thread
        self.start_daemon_thread().await;

        self.waiting_stop().await;
    }

    // start GRPC && HTTP Server
    async fn start_server(&self) {
        let ip = format!("{}:{}", self.config.addr, self.config.port)
            .parse()
            .unwrap();
        self.server_runtime.spawn(async move {
            info_meta(&format!(
                "RobustMQ StorageEngine Grpc Server start success. bind addr:{}",
                ip
            ));

            let service_handler = StorageService::new();

            Server::builder()
                .add_service(StorageEngineServiceServer::new(service_handler))
                .serve(ip)
                .await
                .unwrap();
        });
    }

    // Start Daemon Thread
    async fn start_daemon_thread(&self) {}

    // Wait for the service process to stop
    async fn waiting_stop(&self) {
        loop {
            signal::ctrl_c().await.expect("failed to listen for event");
            match self.stop_send.send(true) {
                Ok(_) => {
                    info_meta("When ctrl + c is received, the service starts to stop");
                    self.stop_server().await;
                    break;
                }
                Err(_) => {}
            }
        }
    }
    async fn stop_server(&self) {
        // unregister node
        // unregister_storage_engine_node(self.config.clone()).await;
    }
}
