use raft_proto::eraftpb::Message as RaftCoreMessage;
use tokio::sync::oneshot::Sender;

pub enum RaftResponseMesage {}
pub enum RaftMessage {
    // Received a message from another node
    Raft(RaftCoreMessage),

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan: Sender<RaftResponseMesage>,
    },
}
