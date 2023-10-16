use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, str::FromStr};

use axum::{routing::{get, post}, Router};

const ROUTE_ROOT: &str = "/";
const ROUTE_METRICS: &str = "/metrics";

mod prometheus;
mod welcome;

pub fn start(addr: &String, port: Option<u16>, worker_threads: usize) {

    let runtime = tokio::runtime::Builder::new_current_thread()
        .worker_threads(worker_threads)
        .thread_name("admin-http")
        .enable_io()
        .build()
        .unwrap();

    runtime.block_on(async {
        let app = Router::new()
                .route(ROUTE_METRICS, get(prometheus::handler))
                .route(ROUTE_ROOT, get(welcome::handler));

        let ip = SocketAddr::from_str(&format!("{}:{}",addr,port.unwrap())).unwrap();
        axum::Server::bind(&ip).serve(app.into_make_service()).await.unwrap();
    })
} 


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
