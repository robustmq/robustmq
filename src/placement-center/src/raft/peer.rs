// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clients::{placement::placement::call::send_raft_message, poll::ClientPool};
use log::{debug, error, info};
use protocol::placement_center::generate::placement::SendRaftMessageRequest;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

pub struct PeersManager {
    peer_message_recv: mpsc::Receiver<PeerMessage>,
    client_poll: Arc<ClientPool>,
}

impl PeersManager {
    pub fn new(
        peer_message_recv: mpsc::Receiver<PeerMessage>,
        client_poll: Arc<ClientPool>,
    ) -> PeersManager {
        let pm = PeersManager {
            peer_message_recv,
            client_poll,
        };
        return pm;
    }

    pub async fn start(&mut self) {
        info!(
            "{}",
            "Starts the thread that sends Raft messages to other nodes"
        );
        loop {
            if let Some(data) = self.peer_message_recv.recv().await {
                let addr = data.to;
                let request = SendRaftMessageRequest { message: data.data };
                match send_raft_message(self.client_poll.clone(), vec![addr.clone()], request).await
                {
                    Ok(_) => debug!("Send Raft message to node {} Successful.", addr),
                    Err(e) => error!(
                        "Failed to send data to {}, error message: {}",
                        addr,
                        e.to_string()
                    ),
                }
            }
        }
    }
}
