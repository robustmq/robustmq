// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
    routing::post,
    Router,
};
use common_config::config::BrokerConfig;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::HttpState;

pub const LOGIN_PATH: &str = "/api/v1/login";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    code: i32,
    data: T,
    message: String,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.code == 0 {
            StatusCode::OK
        } else {
            StatusCode::from_u16(self.code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        };
        (status, Json(self)).into_response()
    }
}

fn ok<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        data,
        message: "success".to_string(),
    }
}

pub fn auth_router() -> Router<Arc<HttpState>> {
    Router::new().route(LOGIN_PATH, post(login_handler))
}

pub async fn login_handler(
    State(state): State<Arc<HttpState>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    let config = common_config::broker::broker_config();
    let admin = &config.admin;

    if req.username != admin.username || req.password != admin.password {
        return ApiResponse {
            code: 401,
            data: (),
            message: "Invalid username or password".to_string(),
        }
        .into_response();
    }

    match generate_token(config) {
        Ok((token, expires_in)) => {
            let _ = state; // state available for future use
            ok(LoginResponse { token, expires_in }).into_response()
        }
        Err(e) => ApiResponse {
            code: 500,
            data: (),
            message: format!("Failed to generate token: {e}"),
        }
        .into_response(),
    }
}

pub fn generate_token(config: &BrokerConfig) -> Result<(String, u64), jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ttl_secs = config.admin.token_ttl_hours * 3600;
    let claims = Claims {
        sub: config.admin.username.clone(),
        iat: now,
        exp: now + ttl_secs,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.admin.jwt_secret.as_bytes()),
    )?;
    Ok((token, ttl_secs))
}

pub fn verify_token(
    token: &str,
    config: &BrokerConfig,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.admin.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Returns true if the request originates from loopback (127.0.0.1 / ::1).
fn is_loopback(addr: &SocketAddr) -> bool {
    addr.ip().is_loopback()
}

/// Auth middleware: loopback requests bypass auth; others must carry a valid Bearer token.
pub async fn auth_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Skip auth for loopback (local CLI / curl usage)
    if is_loopback(&addr) {
        return next.run(request).await;
    }

    let _ = state; // available for future token revocation list

    // Extract Bearer token
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => {
            return ApiResponse {
                code: 401,
                data: (),
                message: "Missing Authorization header".to_string(),
            }
            .into_response()
        }
    };

    let config = common_config::broker::broker_config();
    match verify_token(token, config) {
        Ok(_) => next.run(request).await,
        Err(_) => ApiResponse {
            code: 401,
            data: (),
            message: "Invalid or expired token".to_string(),
        }
        .into_response(),
    }
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn successful_login_response_returns_http_ok_with_application_code_zero() {
        let response = ok(LoginResponse {
            token: "test-token".to_string(),
            expires_in: 3600,
        })
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&body).unwrap(),
            json!({
                "code": 0,
                "data": { "token": "test-token", "expires_in": 3600 },
                "message": "success"
            })
        );
    }

    #[tokio::test]
    async fn authentication_errors_preserve_http_status_and_body() {
        for (code, message) in [
            (401, "Invalid username or password"),
            (401, "Missing Authorization header"),
            (401, "Invalid or expired token"),
            (500, "Failed to generate token: test error"),
        ] {
            let response = ApiResponse {
                code,
                data: (),
                message: message.to_string(),
            }
            .into_response();

            assert_eq!(response.status().as_u16(), code as u16);
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            assert_eq!(
                serde_json::from_slice::<Value>(&body).unwrap(),
                json!({ "code": code, "data": null, "message": message })
            );
        }
    }
}
