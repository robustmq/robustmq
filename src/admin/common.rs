pub async fn metrics_handler() -> String {
    return crate::SERVER_METRICS.gather();
}

pub async fn welcome_handler() -> &'static str {
    crate::SERVER_METRICS.set_server_status_stop();
    "Welcome to RobustMQ"
}