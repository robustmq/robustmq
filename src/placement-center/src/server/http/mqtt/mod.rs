use super::{
    path_create, path_delete, path_get, path_list, path_update, server::HttpServerState, v1_path,
};
use axum::routing::{delete, get, post, put};
use axum::Router;
use cluster::{cluster_get, cluster_list, cluster_set};
use user::{user_create, user_delete, user_get, user_list, user_update};

pub mod cluster;
pub mod user;

pub const ROUTE_USER: &str = "/user";
pub const ROUTE_CLUSTER: &str = "/cluster";

pub fn mqtt_routes() -> Router<HttpServerState> {
    return Router::new()
        // user
        .route(&v1_path(&path_get(ROUTE_USER)), get(user_get))
        .route(&v1_path(&path_create(ROUTE_USER)), post(user_create))
        .route(&v1_path(&path_update(ROUTE_USER)), put(user_update))
        .route(&v1_path(&path_delete(ROUTE_USER)), delete(user_delete))
        .route(&v1_path(&path_list(ROUTE_USER)), get(user_list))
        // cluster
        .route(&v1_path(&path_get(ROUTE_CLUSTER)), get(cluster_get))
        .route(&v1_path(&path_update(ROUTE_CLUSTER)), put(cluster_set))
        .route(&v1_path(&path_list(ROUTE_CLUSTER)), get(cluster_list));
}
