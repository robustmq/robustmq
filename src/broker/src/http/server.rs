use super::route;
use common::{log::info, metrics::broker::metrics_http_broker_running};
use std::net::SocketAddr;

pub struct HttpServer {
    ip: SocketAddr,
}

impl HttpServer {
    
    pub fn new(ip: SocketAddr) -> Self {
        return Self { ip };
    }

    pub async fn start(&self) {
        let ip = self.ip.clone();
        let app = route::routes();
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        info(&format!(
            "RobustMQ Broker HTTP Server start success. bind addr:{}",
            ip
        ));
        metrics_http_broker_running();
        axum::serve(listener, app).await.unwrap();
    }
}
