use crate::{client::send_raft_message, Node};
use ahash::AHashMap;
use common::log::{error_meta, info_meta};
use tokio::sync::broadcast::{self, Sender};

#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PeerManager {
    pub peers: AHashMap<u64, Node>,
    pub peer_message_send: Sender<PeerMessage>,
    pub stop_send: Sender<bool>,
}

impl PeerManager {
    pub fn new(peer_message_send: Sender<PeerMessage>, stop_send: Sender<bool>) -> PeerManager {
        let peers = AHashMap::new();
        let pm = PeerManager {
            peers,
            peer_message_send,
            stop_send,
        };
        return pm;
    }

    // Add Meta node
    pub fn add_peer(&mut self, id: u64, node: Node) {
        self.peers.insert(id, node);
    }

    // Add Meta node
    pub fn remove_peer(&mut self, id: u64) {
        self.peers.remove(&id);
    }

    pub fn start(&mut self) {
        let mut stop_recv = self.stop_send.subscribe();
        let mut peer_message_recv = self.peer_message_send.subscribe();
        tokio::spawn(async move {
            info_meta(&format!(
                "Starts the thread that sends Raft messages to other nodes"
            ));
            loop {
                println!("{}",111);
                match stop_recv.try_recv() {
                    Ok(_) => break,
                    Err(_) => {}
                }
                println!("{}",222);
                match peer_message_recv.recv().await {
                    Ok(data) => {
                        let addr = data.to;
                        let data = data.data;
                        match send_raft_message(&addr, data).await {
                            Ok(_) => info_meta(&format!(
                                "Send Raft message to node {} Successful.",
                                addr
                            )),
                            Err(e) => error_meta(&e.to_string()),
                        }
                    }
                    Err(e) => {
                        error_meta(&e.to_string());
                    }
                }
                println!("{}",333);
            }
        });
    }

    pub async fn send_message(&mut self, message: PeerMessage) {
        match self.peer_message_send.send(message) {
            Ok(_) => {}
            Err(e) => error_meta(&format!(
                "Failed to write Raft Message to send queue with error message: {:?}",
                e.to_string()
            )),
        };
    }

    pub fn get_node_by_id(&self, id: u64) -> Option<&Node> {
        self.peers.get(&id)
    }

    pub async fn stop(&self) {
        match self.stop_send.send(true) {
            Ok(_) => {}
            Err(e) => error_meta(&format!(
                "Failed to stop sending Raft Message, error message: {}",
                e.to_string()
            )),
        }
    }
}
