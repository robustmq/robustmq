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

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{Json,IntoResponse,Response}
};
use serde_json::json;

#[derive(Error, Debug, PartialEq)]
pub enum HttpError {
    #[error("Not found for {0}")]
    NotFound(String)
}

impl IntoResponse for HttpError {
   
    fn into_response(self) -> Response{
        let (status, error_message) = match self {
            HttpError::NotFound(msg) => {
                (StatusCode::NOT_FOUND,msg)
            }
        };
        let body = Json(json!({
            "error": format!("invalid path {}", error_message),
        }));
    
        (status, body).into_response()
    }
   
}
