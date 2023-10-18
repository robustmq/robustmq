use std::{
    net::SocketAddr,
    str::FromStr,
    thread::{self, JoinHandle},
};

use axum::{
    routing::{get, post},
    Router,
};

const ROUTE_ROOT: &str = "/";
const ROUTE_METRICS: &str = "/metrics";
const ROUTE_MANAGEMENT_API_OVERVIEW: &str = "/api/overview";
const ROUTE_MANAGEMENT_API_CLUSTER: &str = "/api/cluster-name";
const ROUTE_MANAGEMENT_API_NODES: &str = "/api/nodes";
const ROUTE_MANAGEMENT_API_NODE_NAME: &str = "/api/nodes/name";

mod management_api;
mod prometheus;
mod welcome;

pub fn start(addr: String, port: Option<u16>, worker_threads: usize) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .worker_threads(worker_threads)
            .thread_name("admin-http")
            .enable_io()
            .build()
            .unwrap();

        runtime.block_on(async {
        
            let app = router_construct();

            let ip = SocketAddr::from_str(&format!("{}:{}", addr, port.unwrap())).unwrap();
            axum::Server::bind(&ip)
                .serve(app.into_make_service())
                .await
                .unwrap();
        })
    });
    return handle;
}

fn router_construct() -> Router {
    // define Management API routes separately
    let management_api_routes = Router::new()
        .route(
            ROUTE_MANAGEMENT_API_OVERVIEW,
            get(management_api::api_overview_get_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_CLUSTER,
            get(management_api::api_cluster_get_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_CLUSTER,
            post(management_api::api_cluster_post_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_NODES,
            get(management_api::api_nodes_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_NODE_NAME,
            get(management_api::api_node_name_handler),
        );

    let other_routes = Router::new()
        .route(ROUTE_METRICS, get(prometheus::handler))
        .route(ROUTE_ROOT, get(welcome::handler));

    let router = Router::new()
        .merge(management_api_routes)
        .merge(other_routes);
    router

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
