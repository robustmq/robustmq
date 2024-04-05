use crate::{metadata::subscriber::Subscriber, server::tcp::packet::ResponsePackage};
use flume::Sender;
use protocol::mqtt::Unsubscribe;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SubScribeManager {
    pub subscribe_list: HashMap<u64, Subscriber>,
    response_queue_sx: Sender<ResponsePackage>,
}

impl SubScribeManager {
    pub fn new(response_queue_sx: Sender<ResponsePackage>) -> Self {
        return SubScribeManager {
            subscribe_list: HashMap::new(),
            response_queue_sx,
        };
    }

    pub fn add_subscribe(&mut self, connect_id: u64, subscriber: Subscriber) {
        self.subscribe_list.insert(connect_id, subscriber);
    }

    pub fn remove_subscribe(&mut self, connect_id: u64, un_subscribe: Option<Unsubscribe>) {
        self.subscribe_list.remove(&connect_id);
    }

    pub fn start_connect_id_subscribe(&self) {
        loop {
            
        }
    }
}
