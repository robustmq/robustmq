use crate::metadata::subscriber::Subscriber;
use protocol::mqtt::Unsubscribe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct SubScribeManager {
    pub subscribe_list: HashMap<u64, Subscriber>,
}

impl SubScribeManager {
    pub fn new() -> Self {
        return SubScribeManager {
            subscribe_list: HashMap::new(),
        };
    }

    pub fn add_subscribe(&mut self, connect_id: u64, subscriber: Subscriber) {
        self.subscribe_list.insert(connect_id, subscriber);
    }

    pub fn remove_subscribe(&mut self, connect_id: u64, un_subscribe: Option<Unsubscribe>) {
        self.subscribe_list.remove(&connect_id);
    }
}
