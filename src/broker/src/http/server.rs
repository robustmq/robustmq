use common::log::info;

use super::route;

use std::{net::SocketAddr, str::FromStr};

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> Self {
        return Self {};
    }

    pub async fn start(&self) {
        let ip = "127.0.0.1:9987";
        tokio::spawn(async move {
            let ip_addr = SocketAddr::from_str(&ip).unwrap();
            let app = route::routes();
            info(&format!("RobustMQ Broker HTTP Server start success. bind addr:{}", ip));
            // axum::Server::bind(&ip_addr)
            //     .serve(app.into_make_service())
            //     .await
            //     .unwrap();
        });
    }
}
