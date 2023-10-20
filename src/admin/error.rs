use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{Json,IntoResponse,Response}
};
use serde_json::json;

#[derive(Error, Debug, PartialEq)]
pub enum ApiError {
    #[error("Not found for {0}")]
    NotFound(String)
}

impl IntoResponse for ApiError {
   
    fn into_response(self) -> Response{
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND,msg)
            }
        };
        let body = Json(json!({
            "error": format!("invalid path {}", error_message),
        }));
    
        (status, body).into_response()
    }
   
}
