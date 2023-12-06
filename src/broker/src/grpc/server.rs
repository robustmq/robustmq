use super::services::GrpcBrokerServices;
use common::log::info;
use protocol::robust::broker::broker_service_server::BrokerServiceServer;
use std::net::SocketAddr;
use tonic::transport::Server;

pub struct GrpcServer {
    addr: SocketAddr,
}

impl GrpcServer {
    pub fn new(addr: SocketAddr) -> Self {
        return Self { addr };
    }
    pub async fn start(&self) {
        info(&format!(
            "RobustMQ Broker Grpc Server start success. bind addr:{}",
            self.addr
        ));
        let service_handler = GrpcBrokerServices::new();
        Server::builder()
            .add_service(BrokerServiceServer::new(service_handler))
            .serve(self.addr)
            .await
            .unwrap();
    }
}
