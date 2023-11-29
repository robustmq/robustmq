use prometheus::{Encoder, TextEncoder};

pub fn dump_metrics() -> Result<String, String> {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let mf = prometheus::gather();
    encoder
        .encode(&mf, &mut buffer)
        .map_err(|_| "Encode Metrics failed".to_string())?;
    String::from_utf8(buffer).map_err(|e| e.to_string())
}
