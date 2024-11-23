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
        let stream = Framed::new(socket, JournalServerCodec::new());
        Ok(())
    }
}
