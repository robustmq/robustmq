use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MQTTNodeExtend {
    pub grpc_addr: String,
    pub http_addr: String,
    pub mqtt4_addr: String,
    pub mqtt4s_addr: String,
    pub mqtt5_addr: String,
    pub mqtt5s_addr: String,
    pub websocket_addr: String,
    pub websockets_addr: String,
}
