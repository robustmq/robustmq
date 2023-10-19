use axum::{routing::get, Router};
use crate::admin::management_api;
use crate::admin::prometheus;
use crate::admin::welcome;

const ROUTE_ROOT: &str = "/";
const ROUTE_METRICS: &str = "/metrics";
const ROUTE_MANAGEMENT_API_OVERVIEW: &str = "/api/overview";
const ROUTE_MANAGEMENT_API_CLUSTER: &str = "/api/cluster-name";
const ROUTE_MANAGEMENT_API_NODES: &str = "/api/nodes";
const ROUTE_MANAGEMENT_API_NODE_NAME: &str = "/api/nodes/name";

pub fn router_construct() -> Router {
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