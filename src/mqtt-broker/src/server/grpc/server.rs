use super::services::GrpcBrokerServices;
use common_base::{log::info, metrics::broker::metrics_grpc_broker_running};
use protocol::mqtt_server::mqtt::mqtt_broker_service_server::MqttBrokerServiceServer;
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
        info(format!(
            "RobustMQ Broker Grpc Server start success. bind addr:{}",
            self.addr
        ));
        metrics_grpc_broker_running();
        let service_handler = GrpcBrokerServices::new();
        Server::builder()
            .add_service(MqttBrokerServiceServer::new(service_handler))
            .serve(self.addr)
            .await
            .unwrap();
    }
}
