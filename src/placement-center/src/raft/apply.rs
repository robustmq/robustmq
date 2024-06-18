use bincode::serialize;
use common_base::errors::RobustMQError;
use raft::eraftpb::ConfChange;
use raft::eraftpb::Message as raftPreludeMessage;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::time::Duration;
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

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    // Cluster
    ClusterRegisterNode,
    ClusterUngisterNode,
    ClusterSetResourceConfig,
    ClusterDeleteResourceConfig,

    // Journal
    JournalCreateShard,
    JournalDeleteShard,
    JournalCreateSegment,
    JournalDeleteSegment,

    // kv
    KvSet,
    KvDelete,

    // mqtt
    MQTTCreateUser,
    MQTTDeleteUser,
    MQTTCreateTopic,
    MQTTDeleteTopic,
    MQTTSetTopicRetainMessage,
    MQTTCreateSession,
    MQTTDeleteSession,
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
        write!(f, "({:?}, {:?})", self.data_type, self.value)
    }
}
pub struct RaftMachineApply {
    raft_status_machine_sender: tokio::sync::mpsc::Sender<RaftMessage>,
}

impl RaftMachineApply {
    pub fn new(raft_sender: tokio::sync::mpsc::Sender<RaftMessage>) -> Self {
        return RaftMachineApply {
            raft_status_machine_sender: raft_sender,
        };
    }

    pub async fn transfer_leader(&self, node_id: u64) -> Result<(), RobustMQError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        return self
            .apply_raft_status_machine_message(
                RaftMessage::TransferLeader {
                    node_id: node_id,
                    chan: sx,
                },
                "transfer_leader".to_string(),
                rx,
            )
            .await;
    }

    pub async fn apply_propose_message(
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

    pub async fn apply_raft_message(
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

    pub async fn apply_conf_raft_message(
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
