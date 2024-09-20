use axum::extract::State;
use common_base::http_response::success_response;

use super::server::HttpServerState;

pub async fn connection_list(State(_): State<HttpServerState>) -> String {
    return success_response("data");
}
