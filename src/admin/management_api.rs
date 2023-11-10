
use axum::{
    extract::Path,
    response::Json
};

use serde_json::{json, Value};

use crate::admin::error::HttpError;

///api_get_handler is used to handle the API request which doesn't contain any query parameters or 
/// specical objects

pub async fn api_get_handler(Path(path) : Path<String>) -> Result<Json<Value>, HttpError> {
    println!("path is : {:#?}", path);
    let api_path = path.to_owned(); 
    match &api_path as &str {
        "overview" => Ok(Json(json!({"API": " return API overview"}))),
        "cluster_name" =>Ok(Json(json!({"API": " return cluster name"}))),
        _ => Err(HttpError::NotFound(api_path)),
    }
}

pub async fn api_cluster_get_handler() -> Json<Value> {
    let cluster_name = "production_cluster";
    Json(json!({ "cluster_name": cluster_name }))
}

pub async fn api_nodes_handler() -> Json<Value>  {
    let nodes = "cluster nodes: node 1, node 2, node 3";
    Json(json!({ "cluster_name": nodes }))
}

pub async fn api_node_name_handler(Path(node_id): Path<u32>) -> Json<Value> {

    Json(json!({ "node_name" : node_id }))

}