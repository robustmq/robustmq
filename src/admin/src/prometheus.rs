use metrics::SERVER_METRICS;

pub async fn handler() -> String {
    let mtr = SERVER_METRICS.gather();    
    return mtr;
}