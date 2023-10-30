use super::proto::meta::{
    meta_service_server::{MetaService, MetaServiceServer},
    HelloReply, HelloRequest,
};
use crate::{config::meta::MetaConfig, log};
use tokio::runtime::Runtime;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct MetaServiceHandler {}

#[tonic::async_trait]
impl MetaService for MetaServiceHandler {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

pub struct GrpcServer<'a> {
    meta_config: &'a MetaConfig,
}

impl<'a> GrpcServer<'a> {
    pub fn new(meta_config: &'a MetaConfig) -> Self {
        return GrpcServer {
            meta_config: meta_config,
        };
    }

    pub fn start(&self) -> Runtime {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(self.config.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();
        let ip = format!(
            "{}:{}",
            self.meta_config.addr,
            self.meta_config.port.unwrap()
        )
        .parse()
        .unwrap();

        log::info(&format!("MetaSerice listening on {}", &ip));
        runtime.spawn(async move {
            let meta_service_handle = MetaServiceHandler::default();
            Server::builder()
                .add_service(MetaServiceServer::new(meta_service_handle))
                .serve(ip)
                .await
                .unwrap();
        });
        return runtime;
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use tokio::runtime::Runtime;
    use tonic_build;

    use crate::meta::proto::meta::{meta_service_client::MetaServiceClient, HelloRequest};
    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/meta/proto") // you can change the generated code's location
            .compile(
                &["src/meta/proto/meta.proto"],
                &["src/meta/proto/"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }

    #[test]
    fn grpc_client() {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(self.config.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();

        runtime.spawn(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(HelloRequest {
                name: "lobo11".into(),
            });

            let response = client.say_hello(request).await.unwrap();

            println!("response={:?}", response);
        });

        loop {
            thread::sleep(Duration::from_secs(300));
        }
    }
}
