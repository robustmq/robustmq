use std::fmt;
use std::time::Duration;

use bincode::serialize;
use common::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::placement::CreateSegmentRequest;
use protocol::placement_center::placement::CreateShardRequest;
use protocol::placement_center::placement::DeleteSegmentRequest;
use protocol::placement_center::placement::DeleteShardRequest;
use protocol::placement_center::placement::RegisterNodeRequest;
use protocol::placement_center::placement::UnRegisterNodeRequest;
use raft::eraftpb::ConfChange;
use raft::eraftpb::Message as raftPreludeMessage;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;
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

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan: Sender<RaftResponseMesage>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    RegisterNode,
    UngisterNode,
    CreateShard,
    DeleteShard,
    CreateSegment,
    DeleteSegment,
}

impl fmt::Display for StorageDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageDataType::RegisterNode => {
                write!(f, "RegisterNode")
            }
            StorageDataType::UngisterNode => {
                write!(f, "UngisterNode")
            }
            StorageDataType::CreateShard => {
                write!(f, "CreateShard")
            }
            StorageDataType::DeleteShard => {
                write!(f, "DeleteShard")
            }
            StorageDataType::CreateSegment => {
                write!(f, "CreateSegment")
            }
            StorageDataType::DeleteSegment => {
                write!(f, "DeleteSegment")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Vec<u8>) -> StorageData {
        return StorageData {
            data_type,
            value: value,
        };
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.data_type, self.value)
    }
}
pub struct PlacementCenterStorage {
    raft_status_machine_sender: tokio::sync::mpsc::Sender<RaftMessage>,
}

impl PlacementCenterStorage {
    pub fn new(raft_sender: tokio::sync::mpsc::Sender<RaftMessage>) -> Self {
        return PlacementCenterStorage {
            raft_status_machine_sender: raft_sender,
        };
    }

    //
    pub async fn save_node(&self, data: RegisterNodeRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::RegisterNode,
            RegisterNodeRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "register_node".to_string())
            .await;
    }

    pub async fn delete_node(&self, data: UnRegisterNodeRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::UngisterNode,
            UnRegisterNodeRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "un_register_node".to_string())
            .await;
    }

    pub async fn save_shard(&self, data: CreateShardRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::CreateShard,
            CreateShardRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "create_shard".to_string())
            .await;
    }

    pub async fn delete_shard(&self, data: DeleteShardRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::DeleteShard,
            DeleteShardRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "delete_shard".to_string())
            .await;
    }

    pub async fn create_segment(&self, data: CreateSegmentRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::CreateSegment,
            CreateSegmentRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "create_segment".to_string())
            .await;
    }

    pub async fn delete_segment(&self, data: DeleteSegmentRequest) -> Result<(), RobustMQError> {
        let data = StorageData::new(
            StorageDataType::DeleteSegment,
            DeleteSegmentRequest::encode_to_vec(&data),
        );
        return self
            .apply_propose_message(data, "delete_segment".to_string())
            .await;
    }

    pub async fn save_raft_message(
        &self,
        message: raftPreludeMessage,
    ) -> Result<(), RobustMQError> {
        return self
            .apply_raft_message(message, "send_raft_message".to_string())
            .await;
    }

    pub async fn save_conf_raft_message(&self, change: ConfChange) -> Result<(), RobustMQError> {
        return self
            .apply_conf_raft_message(change, "send_conf_raft_message".to_string())
            .await;
    }

    pub fn get_raft_status_machine_sender(&self) -> tokio::sync::mpsc::Sender<RaftMessage> {
        return self.raft_status_machine_sender.clone();
    }

    //
    async fn apply_propose_message(
        &self,
        data: StorageData,
        action: String,
    ) -> Result<(), RobustMQError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return self
            .apply_raft_status_machine_message(
                RaftMessage::Propose {
                    data: serialize(&data).unwrap(),
                    chan: sx,
                },
                action,
                rx,
            )
            .await;
    }

    //
    async fn apply_raft_message(
        &self,
        message: raftPreludeMessage,
        action: String,
    ) -> Result<(), RobustMQError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return self
            .apply_raft_status_machine_message(
                RaftMessage::Raft {
                    message: message,
                    chan: sx,
                },
                action,
                rx,
            )
            .await;
    }

    //
    async fn apply_conf_raft_message(
        &self,
        change: ConfChange,
        action: String,
    ) -> Result<(), RobustMQError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return self
            .apply_raft_status_machine_message(
                RaftMessage::ConfChange { change, chan: sx },
                action,
                rx,
            )
            .await;
    }

    async fn apply_raft_status_machine_message(
        &self,
        message: RaftMessage,
        action: String,
        rx: Receiver<RaftResponseMesage>,
    ) -> Result<(), RobustMQError> {
        let _ = self.raft_status_machine_sender.send(message).await;
        if !self.wait_recv_chan_resp(rx).await {
            return Err(RobustMQError::MetaLogCommitTimeout(action));
        }
        return Ok(());
    }

    //
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
