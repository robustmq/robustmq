use common_base::error::common::CommonError;
use dashmap::DashMap;
use protocol::journal_server::codec::JournalServerCodec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct ClientConnection {
    conn: Framed<TcpStream, JournalServerCodec>,
    last_active_time: u64,
}
pub struct NodeConnection {
    // connection_type, Conn
    connections: DashMap<String, ClientConnection>,
}
pub struct ConnectionManager {
    node_conns: DashMap<String, NodeConnection>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let node_conns = DashMap::with_capacity(2);
        ConnectionManager { node_conns }
    }

    pub async fn open(&self, node_id: String) -> Result<(), CommonError> {
        
        let socket = TcpStream::connect("").await?;
        let mut stream = Framed::new(socket, JournalServerCodec::new());
        Ok(())
    }
}
