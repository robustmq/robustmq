use metrics::SERVER_METRICS;
pub async fn handler() -> &'static str {
    SERVER_METRICS.set_server_status_stop();
    "Welcome to RobustMQ"
}