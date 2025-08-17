use crate::packet::{ResponsePackage, RobustMQPacket};
use axum::async_trait;
use metadata_struct::connection::NetworkConnection;
use std::{net::SocketAddr, sync::Arc};

#[async_trait]
pub trait Command {
    async fn apply(
        &self,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        packet: RobustMQPacket,
    ) -> Option<ResponsePackage>;
}
pub type ArcCommandAdapter = Arc<Box<dyn Command + Send + Sync>>;
