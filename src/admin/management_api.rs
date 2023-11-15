/*
 * Copyright (c) 2023 robustmq team 
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *     http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


use axum::{
    extract::Path,
    response::Json
};

use serde_json::{json, Value};
use crate::admin::error::ApiError;

///api_get_handler is used to handle the API request which doesn't contain any query parameters or 
/// specical objects

pub async fn api_get_handler(Path(path) : Path<String>) -> Result<Json<Value>, ApiError> {
    println!("path is : {:#?}", path);
    let api_path = path.to_owned(); 
    match &api_path as &str {
        "overview" => Ok(Json(json!({"API": " return API overview"}))),
        "cluster_name" =>Ok(Json(json!({"API": " return cluster name"}))),
        _ => Err(ApiError::NotFound(api_path)),
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