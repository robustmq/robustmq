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

use std::time::Duration;

use bincode::serialize;
use openraft::raft::ClientWriteResponse;
use openraft::Raft;
use raft::eraftpb::{ConfChange, Message as RaftPreludeMessage};
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::time::timeout;

use crate::core::error::PlacementCenterError;
use crate::raft::raftv2::typeconfig::TypeConfig;
use crate::route::data::StorageData;

pub enum RaftResponseMessage {
    Success,
    Fail,
}
pub enum RaftMessage {
    ConfChange {
        change: ConfChange,
        chan: Sender<RaftResponseMessage>,
    },

    // Received a message from another node
    Raft {
        // We put the message in the box to avoid having a large enum variant
        message: Box<RaftPreludeMessage>,
        chan: Sender<RaftResponseMessage>,
    },

    TransferLeader {
        node_id: u64,
        chan: Sender<RaftResponseMessage>,
    },

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan: Sender<RaftResponseMessage>,
    },
}

#[derive(PartialEq, Eq)]
pub enum ClusterRaftModel {
    V1,
    V2,
}

pub struct RaftMachineApply {
    raft_status_machine_sender: tokio::sync::mpsc::Sender<RaftMessage>,
    pub openraft_node: Raft<TypeConfig>,
    pub model: ClusterRaftModel,
}

impl RaftMachineApply {
    pub fn new(
        raft_sender: tokio::sync::mpsc::Sender<RaftMessage>,
        openraft_node: Raft<TypeConfig>,
        model: ClusterRaftModel,
    ) -> Self {
        RaftMachineApply {
            raft_status_machine_sender: raft_sender,
            openraft_node,
            model,
        }
    }

    pub async fn client_write(
        &self,
        data: StorageData,
    ) -> Result<Option<ClientWriteResponse<TypeConfig>>, PlacementCenterError> {
        if self.model == ClusterRaftModel::V1 {
            let action = format!("{:?}", data.data_type);
            match self.raftv1_write(data, action).await {
                Ok(()) => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        if self.model == ClusterRaftModel::V2 {
            match self.raftv2_write(data).await {
                Ok(data) => return Ok(Some(data)),
                Err(e) => return Err(e),
            }
        }

        panic!("raft cluster mode is not available, optional :V1,V2");
    }

    async fn raftv1_write(
        &self,
        data: StorageData,
        action: String,
    ) -> Result<(), PlacementCenterError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        self.apply_raft_status_machine_message(
            RaftMessage::Propose {
                data: serialize(&data).unwrap(),
                chan: sx,
            },
            action,
            rx,
        )
        .await
    }

    async fn raftv2_write(
        &self,
        data: StorageData,
    ) -> Result<ClientWriteResponse<TypeConfig>, PlacementCenterError> {
        let resp = timeout(
            Duration::from_secs(10),
            self.openraft_node.client_write(data),
        )
        .await?;
        Ok(resp?)
    }

    pub async fn transfer_leader(&self, node_id: u64) -> Result<(), PlacementCenterError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        self.apply_raft_status_machine_message(
            RaftMessage::TransferLeader { node_id, chan: sx },
            "transfer_leader".to_string(),
            rx,
        )
        .await
    }

    pub async fn apply_raft_message(
        &self,
        message: RaftPreludeMessage,
        action: String,
    ) -> Result<(), PlacementCenterError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        self.apply_raft_status_machine_message(
            RaftMessage::Raft {
                message: Box::new(message),
                chan: sx,
            },
            action,
            rx,
        )
        .await
    }

    pub async fn apply_conf_raft_message(
        &self,
        change: ConfChange,
        action: String,
    ) -> Result<(), PlacementCenterError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        self.apply_raft_status_machine_message(
            RaftMessage::ConfChange { change, chan: sx },
            action,
            rx,
        )
        .await
    }

    async fn apply_raft_status_machine_message(
        &self,
        message: RaftMessage,
        action: String,
        rx: Receiver<RaftResponseMessage>,
    ) -> Result<(), PlacementCenterError> {
        let _ = self.raft_status_machine_sender.send(message).await;
        if !self.wait_recv_chan_resp(rx).await {
            return Err(PlacementCenterError::RaftLogCommitTimeout(action));
        }
        Ok(())
    }

    async fn wait_recv_chan_resp(&self, rx: Receiver<RaftResponseMessage>) -> bool {
        let res = timeout(Duration::from_secs(30), async {
            match rx.await {
                Ok(val) => val,
                Err(_) => RaftResponseMessage::Fail,
            }
        });
        res.await.is_ok()
    }
}
