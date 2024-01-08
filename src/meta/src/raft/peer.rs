use crate::client::send_raft_message;
use common::log::{error_meta, info_meta};
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct PeersManager {
    pub peer_message_recv: mpsc::Receiver<PeerMessage>,
}

impl PeersManager {
    pub fn new(
        peer_message_recv: mpsc::Receiver<PeerMessage>,
    ) -> PeersManager {
        let pm = PeersManager {
            peer_message_recv,
        };
        return pm;
    }
    pub async fn start(&mut self) {
        info_meta(&format!(
            "Starts the thread that sends Raft messages to other nodes"
        ));

        while let Some(data) = self.peer_message_recv.recv().await {
            let addr = data.to;
            let data = data.data;
            match send_raft_message(&addr, data).await {
                Ok(_) => info_meta(&format!("Send Raft message to node {} Successful.", addr)),
                Err(e) => error_meta(&e.to_string()),
            }
        }
    }
}
