use axum::Router;
use axum::routing::get;
use crate::admin::management_api;
use crate::admin::common;

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_METRICS: &str = "/metrics";
// pub const ROUTE_MANAGEMENT_API_OVERVIEW: &str = "/api/overview";
pub const ROUTE_MANAGEMENT_API_CLUSTER: &str = "/api/cluster-name";
pub const ROUTE_MANAGEMENT_API_NODES: &str = "/api/nodes";
pub const ROUTE_MANAGEMENT_API_NODE_NAME: &str = "/api/nodes/:node_id";
pub const ROUTE_MANAGEMENT_API_NAME: &str = "/api/:name";


pub fn routes() -> Router {

    let management = Router::new()
        .route(
            ROUTE_MANAGEMENT_API_NAME,
            get(management_api::api_get_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_CLUSTER,
            get(management_api::api_cluster_get_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_NODES,
            get(management_api::api_nodes_handler),
        )
        .route(
            ROUTE_MANAGEMENT_API_NODE_NAME,
            get(management_api::api_node_name_handler),
        );

    let common = Router::new()
        .route(ROUTE_METRICS, get(common::metrics_handler))
        .route(ROUTE_ROOT, get(common::welcome_handler));

    let app = Router::new()
        .merge(management)
        .merge(common);

    return app;
}
