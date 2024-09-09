pub struct SlowMessage {
    client_id: String,
    topic: String,
    time_ms: u16,
    node_info: String,
    create_time: u128,
}

pub fn try_record_slow_message(client_id: String, topic: String, time_ms: u16) {
    
}
