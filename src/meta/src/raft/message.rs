use tokio::sync::oneshot::Sender;
use raft::eraftpb::Message as raftPreludeMessage;

pub enum RaftResponseMesage {}
pub enum RaftMessage {
    
    // Received a message from another node
    Raft(raftPreludeMessage),

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan: Sender<RaftResponseMesage>,
    },
}
