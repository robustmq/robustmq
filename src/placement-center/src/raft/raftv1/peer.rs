// Copyright 2023 RobustMQ Team
//
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

use std::collections::HashMap;
use std::sync::Arc;

use grpc_clients::placement::placement::call::send_raft_message;
use grpc_clients::pool::ClientPool;
use log::{debug, error};
use protocol::placement_center::placement_center_inner::SendRaftMessageRequest;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

pub struct RaftPeersManager {
    peer_message_recv: mpsc::Receiver<PeerMessage>,
    client_poll: Arc<ClientPool>,
    send_thread_num: usize,
    sender_threads: HashMap<usize, Sender<PeerMessage>>,
    stop_send: broadcast::Sender<bool>,
}

impl RaftPeersManager {
    pub fn new(
        peer_message_recv: mpsc::Receiver<PeerMessage>,
        client_poll: Arc<ClientPool>,
        send_thread_num: usize,
        stop_send: broadcast::Sender<bool>,
    ) -> RaftPeersManager {
        RaftPeersManager {
            peer_message_recv,
            client_poll,
            send_thread_num,
            stop_send,
            sender_threads: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        debug!("Raft peer main thread start successfully.");

        for index in 0..self.send_thread_num {
            let (peer_message_send, peer_message_recv) = mpsc::channel::<PeerMessage>(1000);
            self.sender_threads.insert(index, peer_message_send);
            start_child_send_thread(
                index,
                peer_message_recv,
                self.stop_send.clone(),
                self.client_poll.clone(),
            );
        }

        let mut stop_rx = self.stop_send.subscribe();

        let mut child_channel_index = 0;

        loop {
            select! {

                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("Raft peer main thread stopped successfully.");
                            break;
                        }
                    }
                }

                val = self.peer_message_recv.recv()=>{
                    if let Some(data) = val {
                        if child_channel_index >= self.sender_threads.len(){
                            child_channel_index = 0;
                        }

                        if let Some(send_sx) = self.sender_threads.get(&child_channel_index){
                            match send_sx.send(data).await{
                                Ok(()) => {}
                                Err(e) => {
                                    error!("{}",e);
                                }
                            }
                            child_channel_index += 1;
                        }
                    }
                }
            }
        }
    }
}

fn start_child_send_thread(
    index: usize,
    mut recv: Receiver<PeerMessage>,
    stop_send: broadcast::Sender<bool>,
    client_poll: Arc<ClientPool>,
) {
    tokio::spawn(async move {
        debug!("Raft peer child thread {} start successfully.", index);

        let mut stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("Raft peer child thread {} stopped successfully.",index);
                            break;
                        }
                    }
                }

                val = recv.recv()=>{
                    if let Some(data) = val {
                        let addr = data.to;
                        let request = SendRaftMessageRequest { message: data.data };
                        match send_raft_message(client_poll.clone(), vec![addr.clone()], request).await
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
    });
}
