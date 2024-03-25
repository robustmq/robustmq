use common_base::config::storage_engine::storage_engine_conf;

pub mod quic;
pub mod tcp;
pub mod websocket;

pub async fn start_tcp_server() {
    let conf = storage_engine_conf();
    
}

pub async fn start_quic_server() {
    let conf = storage_engine_conf();
}
