use crate::client::send_raft_message;
use common::log::{debug_meta, error_meta, info_meta};
use std::{thread::sleep, time::Duration};

#[derive(Clone)]
pub struct Peer {
    addr: String,
    active: bool,
    queue: Vec<Vec<u8>>,
}

impl Peer {
    pub fn new(addr: String) -> Self {
        let queue = Vec::new();
        let pr = Peer {
            addr,
            active: true,
            queue,
        };
        return pr;
    }
    pub fn push(&mut self, message: Vec<u8>) {
        self.queue.push(message);
    }

    pub async fn start(&mut self) {
        info_meta(&format!(
            "peer node {} The sending thread is started",
            self.addr.clone()
        ));
        loop {
            if !self.active {
                info_meta(&format!(
                    "peer node {} The sending thread is stoped",
                    self.addr.clone()
                ));
                break;
            }
            match self.queue.pop() {
                Some(msg) => match send_raft_message(&self.addr, msg).await {
                    Ok(_) => debug_meta(&format!(
                        "Send Raft message to node {} Successful.",
                        self.addr
                    )),
                    Err(e) => error_meta(&e.to_string()),
                },
                None => sleep(Duration::from_millis(100)),
            }
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }
}
