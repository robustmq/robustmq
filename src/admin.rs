pub fn start(port: Option<u16>) {
    println!("{}",metrics::SERVER_METRICS.gather());
}
