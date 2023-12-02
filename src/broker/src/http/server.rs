use super::route;
use common_log::log::info;
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
            info(&format!("http server start success. bind:{}", ip));
            axum::Server::bind(&ip_addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    }
}
