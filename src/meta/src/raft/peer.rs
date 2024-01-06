use crate::{client::send_raft_message, Node};
use ahash::AHashMap;
use common::log::{debug_meta, error_meta, info_meta};
use tokio::sync::broadcast::{self, Sender};

#[derive(Clone, Debug)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PeerManager {
    pub peers: AHashMap<u64, Node>,
    pub peer_message_send: Sender<PeerMessage>,
    pub stop_send: Sender<u16>,
    pub thread_num: u16,
}

impl PeerManager {
    pub fn new(thread_num: u16) -> PeerManager {
        let (peer_message_send, _) = broadcast::channel::<PeerMessage>(10000);
        let (stop_send, _) = broadcast::channel::<u16>(10000);
        let peers = AHashMap::new();
        let pm = PeerManager {
            peers,
            peer_message_send,
            stop_send,
            thread_num,
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

    pub fn start(&self) {
        for i in 1..=self.thread_num {
            let mut peer_message_rx = self.peer_message_send.subscribe();
            let mut stop_rx = self.stop_send.subscribe();

            tokio::spawn(async move {
                info_meta(&format!("Start sending thread, thread seq:{}", i));
                loop {
                    match stop_rx.try_recv() {
                        Ok(_) => break,
                        Err(_) => {}
                    }
                    match peer_message_rx.recv().await {
                        Ok(data) => {
                            let addr = data.to;
                            let data = data.data;
                            match send_raft_message(&addr, data).await {
                                Ok(_) => debug_meta(&format!(
                                    "Send Raft message to node {} Successful.",
                                    addr
                                )),
                                Err(e) => error_meta(&e.to_string()),
                            }
                        }
                        Err(_) => {}
                    }
                }
            });
        }
    }

    pub fn send_message(&mut self, message: PeerMessage) {
        match self.peer_message_send.send(message) {
            Ok(_) => {}
            Err(e) => error_meta(&format!(
                "Failed to write Raft Message to send queue with error message: {:?}",
                e
            )),
        };
    }

    pub fn get_node_by_id(&self, id: u64) -> Option<&Node> {
        self.peers.get(&id)
    }

    pub fn stop(&self) {
        for i in 1..=self.thread_num {
            match self.stop_send.send(i) {
                Ok(_) => {}
                Err(e) => error_meta(&format!(
                    "Failed to stop sending Raft Message, error message: {}",
                    e
                )),
            }
        }
    }
}
