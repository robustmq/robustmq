use axum::Router;
use axum::routing::get;
use super::meta::cluster_info;

pub const ROUTE_ROOT: &str = "/";


pub fn routes() -> Router {

    let meta = Router::new()
        .route(
            ROUTE_ROOT,
            get(cluster_info),
        );
    
    let app = Router::new()
        .merge(meta);

    return app;
}