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

use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;

use tokio::sync::mpsc;
use tracing::error;
static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, PartialEq, PartialOrd)]
pub enum NetworkConnectionType {
    Tcp,
    Tls,
}

impl fmt::Display for NetworkConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetworkConnectionType::Tcp => "tcp",
                NetworkConnectionType::Tls => "tls",
            }
        )
    }
}

/// a struct that represents a TCP / TLS network connection
#[derive(Clone)]
pub struct NetworkConnection {
    pub connection_type: NetworkConnectionType,
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub connection_stop_sx: Option<mpsc::Sender<bool>>,
}

impl NetworkConnection {
    pub fn new(
        connection_type: NetworkConnectionType,
        addr: SocketAddr,
        connection_stop_sx: Option<mpsc::Sender<bool>>,
    ) -> Self {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        NetworkConnection {
            connection_type,
            connection_id,
            addr,
            connection_stop_sx,
        }
    }

    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    pub async fn stop_connection(&self) {
        if let Some(sx) = self.connection_stop_sx.clone() {
            match sx.send(true).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
}
