use std::collections::HashMap;

use common::errors::RobustMQError;
use mobc::{Manager, Pool};
use protocol::placement_center::placement::placement_center_service_client::PlacementCenterServiceClient;
use tonic::transport::Channel;

pub mod broker_server;
pub mod placement_center;
pub mod storage_engine;

pub struct GrpcConnectionManager {
    pub addr: String,
}

impl GrpcConnectionManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for GrpcConnectionManager {
    type Connection = PlacementCenterServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match PlacementCenterServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => return Err(RobustMQError::TonicTransport(err)),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

pub struct ClientPool {
    pools: HashMap<String, Pool<GrpcConnectionManager>>,
}

impl Clone for ClientPool {
    fn clone(&self) -> Self {
        Self {
            pools: self.pools.clone(),
        }
    }
}

impl ClientPool {
    pub fn new() -> Self {
        let pools = HashMap::new();
        Self { pools }
    }

    pub async fn get(
        &mut self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        if self.pools.contains_key(&addr) {
            let manager = GrpcConnectionManager::new(addr.clone());
            let pool = Pool::builder().max_open(3).build(manager);
            self.pools.insert(addr.clone(), pool.clone());
        }
        if let Some(client) = self.pools.get(&addr) {
            match client.clone().get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::LeaderExistsNotAllowElection);
                }
            };
        }
        return Err(RobustMQError::LeaderExistsNotAllowElection);
    }
}

#[cfg(test)]
mod tests {
    use protocol::placement_center::placement::RegisterNodeRequest;

    use crate::ClientPool;

    #[tokio::test]
    async fn conects() {
        let mut pool = ClientPool::new();
        let addr = "127.0.0.1:2193".to_string();
        match pool.get(addr).await {
            Ok(mut client) => {
                let r = client.register_node(RegisterNodeRequest::default()).await;
            }
            Err(e) => {}
        }
    }
}
