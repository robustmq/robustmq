use axum::Router;
use axum::routing::get;
use super::cluster::cluster_info;
use super::metrics::{metrics_handler, welcome_handler};

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_METRICS: &str = "/metrics";
pub const ROUTE_CLUSTER_INFO: &str = "/cluster-info";


pub fn routes() -> Router {

    let management = Router::new()
        .route(
            ROUTE_CLUSTER_INFO,
            get(cluster_info),
        );

    let common = Router::new()
        .route(ROUTE_METRICS, get(metrics_handler))
        .route(ROUTE_ROOT, get(welcome_handler));

    let app = Router::new()
        .merge(management)
        .merge(common);

    return app;
}