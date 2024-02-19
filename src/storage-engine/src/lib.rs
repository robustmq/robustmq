use std::thread;

use common::{config::storage_engine::StorageEngineConfig, log::info_meta, runtime::create_runtime};
use tonic::transport::Server;

mod index;
mod record;
mod segment;
mod services;
mod shard;
mod storage;
mod v1;
mod v2;

pub struct StorageEngine {
    config: StorageEngineConfig
}

impl StorageEngine {
    pub fn new(config: StorageEngineConfig) -> Self {
        return StorageEngine {
            config
        };
    }

    pub fn start(&self) {
        let mut thread_result = Vec::new();

        // 
        let config = self.config.clone();
        let tcp_thread = thread::Builder::new().name("storage-engine-tcp-thread".to_owned());
        let tcp_thread_join = tcp_thread.spawn(move || {
            let runtime = create_runtime("storage-engine-tcp-runtime", config.runtime_work_threads);

            runtime.spawn(async move {
                let ip = format!("{}:{}", config.addr, config.port).parse().unwrap();

                info_meta(&format!(
                    "RobustMQ StorageEngine Grpc Server start success. bind addr:{}",
                    ip
                ));

                let service_handler = GrpcService::new(
                    cluster_clone,
                    raft_message_send,
                    rocksdb_storage_c,
                    cluster_storage_c,
                );

                Server::builder()
                    .add_service(MetaServiceServer::new(service_handler))
                    .serve(ip)
                    .await
                    .unwrap();
            });

            runtime.block_on(async {
                if stop_recv_c.recv().await.unwrap() {
                    info_meta("TCP and GRPC Server services stop.");
                }
            });
        });
        thread_result.push(tcp_thread_join);
    }
}
