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

use bincode::serialize;
use common_base::error::common::CommonError;
use common_base::error::placement_center::PlacementCenterError;
use openraft::Raft;
use raft::eraftpb::ConfChange;
use raft::eraftpb::Message as raftPreludeMessage;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;

use crate::raftv2::typeconfig::TypeConfig;
use crate::storage::route::data::StorageData;

pub enum RaftResponseMesage {
    Success,
    Fail,
}
pub enum RaftMessage {
    ConfChange {
        change: ConfChange,
        chan: Sender<RaftResponseMesage>,
    },

    // Received a message from another node
    Raft {
        message: raftPreludeMessage,
        chan: Sender<RaftResponseMesage>,
    },

    TransferLeader {
        node_id: u64,
        chan: Sender<RaftResponseMesage>,
    },

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan: Sender<RaftResponseMesage>,
    },
}

pub struct RaftMachineApply {
    raft_status_machine_sender: tokio::sync::mpsc::Sender<RaftMessage>,
    pub openraft_node: Raft<TypeConfig>,
}

impl RaftMachineApply {
    pub fn new(
        raft_sender: tokio::sync::mpsc::Sender<RaftMessage>,
        openraft_node: Raft<TypeConfig>,
    ) -> Self {
        return RaftMachineApply {
            raft_status_machine_sender: raft_sender,
            openraft_node,
        };
    }

    pub async fn raftv2_write() {}

    pub async fn transfer_leader(&self, node_id: u64) -> Result<(), CommonError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::TransferLeader {
                    node_id: node_id,
                    chan: sx,
                },
                "transfer_leader".to_string(),
                rx,
            )
            .await?);
    }

    pub async fn apply_propose_message(
        &self,
        data: StorageData,
        action: String,
    ) -> Result<(), CommonError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::Propose {
                    data: serialize(&data).unwrap(),
                    chan: sx,
                },
                action,
                rx,
            )
            .await?);
    }

    pub async fn apply_raft_message(
        &self,
        message: raftPreludeMessage,
        action: String,
    ) -> Result<(), CommonError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::Raft {
                    message: message,
                    chan: sx,
                },
                action,
                rx,
            )
            .await?);
    }

    pub async fn apply_conf_raft_message(
        &self,
        change: ConfChange,
        action: String,
    ) -> Result<(), CommonError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::ConfChange { change, chan: sx },
                action,
                rx,
            )
            .await?);
    }

    async fn apply_raft_status_machine_message(
        &self,
        message: RaftMessage,
        action: String,
        rx: Receiver<RaftResponseMesage>,
    ) -> Result<(), PlacementCenterError> {
        let _ = self.raft_status_machine_sender.send(message).await;
        if !self.wait_recv_chan_resp(rx).await {
            return Err(PlacementCenterError::RaftLogCommitTimeout(action));
        }
        return Ok(());
    }

    async fn wait_recv_chan_resp(&self, rx: Receiver<RaftResponseMesage>) -> bool {
        let res = timeout(Duration::from_secs(30), async {
            match rx.await {
                Ok(val) => {
                    return val;
                }
                Err(_) => {
                    return RaftResponseMesage::Fail;
                }
            }
        });
        match res.await {
            Ok(_) => return true,
            Err(_) => {
                return false;
            }
        }
    }
}
