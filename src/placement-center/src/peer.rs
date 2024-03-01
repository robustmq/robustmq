use clients::placement_center::PlacementCenterClientManager;
use protocol::placement_center::placement::ClusterType;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Default)]
pub enum InterfaceEnum {
    #[default]
    SEND_RAFT_MESSAGE,
}

#[derive(Debug, Clone, Default)]
pub struct PeerMessage {
    pub to: String,
    pub interface: InterfaceEnum,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct PeersSender {
    pub peer_message_recv: mpsc::Receiver<PeerMessage>,
    placement_center_client: PlacementCenterClientManager,
    placement_center_queue: Vec<PeerMessage>,
    broker_server_queue: Vec<PeerMessage>,
    storage_engine_queue: Vec<PeerMessage>,
}

impl PeersSender {
    pub fn new(peer_message_recv: mpsc::Receiver<PeerMessage>) -> PeersSender {
        let placement_center_client = PlacementCenterClientManager::new();
        let placement_center_queue = Vec::new();
        let broker_server_queue = Vec::new();
        let storage_engine_queue = Vec::new();
        let pm = PeersSender {
            peer_message_recv,
            placement_center_client,
            placement_center_queue,
            broker_server_queue,
            storage_engine_queue,
        };
        return pm;
    }

    pub async fn send_placement_send(&mut self, message: PeerMessage) {
        self.placement_center_queue.push(message);
    }

    pub async fn storage_engine_send(&mut self, message: PeerMessage) {
        self.storage_engine_queue.push(message);
    }

    pub async fn broker_server_send(&mut self, message: PeerMessage) {
        self.broker_server_queue.push(message);
    }

    // pub async fn start_reveive(&mut self) {
    //     loop {
    //         if let Some(data) = self.peer_message_recv.recv().await {
    //             match data.service {
    //                 ClusterType::PlacementCenter => {
    //                     self.placement_center_queue.push(data);
    //                 }
    //                 ClusterType::StorageEngine => {
    //                     self.broker_server_queue.push(data);
    //                 }
    //                 ClusterType::BrokerServer => {
    //                     self.storage_engine_queue.push(data);
    //                 }
    //             }
    //         }
    //     }
    // }

    // pub async fn retry_call() {
    //     let retry_time = 3;
    //     let time = 0;
    //     loop {
    //         if time > retry_time {
    //             break;
    //         }
    //     }

    // match send_raft_message(&addr, data).await {
    //     Ok(_) => debug_meta(&format!("Send Raft message to node {} Successful.", addr)),
    //     Err(e) => error_meta(&format!(
    //         "Failed to send data to {}, error message: {}",
    //         addr,
    //         e.to_string()
    //     )),
    // }
    // }
}
