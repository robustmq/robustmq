use axum::extract::State;
use crate::server::http::server::HttpServerState;

pub async fn user_get(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn user_create(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn user_update(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn user_delete(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}

pub async fn user_list(State(state): State<HttpServerState>) -> String {
    return "".to_string();
}
