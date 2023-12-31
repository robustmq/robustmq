use raft::eraftpb::Message as raftPreludeMessage;
use tokio::sync::oneshot::Sender;

pub enum RaftResponseMesage {
    Success,
    Fail,
}
pub enum RaftMessage {
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
