/*
 * Copyright (c) 2023 RobustMQ Team
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
use crate::http::error::HttpError;
use axum::response::Json;
use serde_json::{json, Value};

pub async fn cluster_info() -> Result<Json<Value>, HttpError> {
    let path = "";
    println!("path is : {:#?}", path);
    let api_path = path.to_owned();
    match &api_path as &str {
        "overview" => Ok(Json(json!({"API": " return API overview"}))),
        "cluster_name" => Ok(Json(json!({"API": " return cluster name"}))),
        _ => Err(HttpError::NotFound(api_path)),
    }
}
