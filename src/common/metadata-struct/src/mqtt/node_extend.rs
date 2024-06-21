use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MQTTNodeExtend {
    pub grpc_addr: String,
    pub http_addr: String,
    pub mqtt_addr: String,
    pub mqtts_addr: String,
    pub websocket_addr: String,
    pub websockets_addr: String,
    pub quic_addr: String,
}
