use crate::ClientPool;
use common_base::errors::RobustMQError;
use mobc::Manager;
use protocol::placement_center::generate::{
    kv::kv_service_client::KvServiceClient,
    placement::placement_center_service_client::PlacementCenterServiceClient,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use super::PlacementCenterInterface;

pub mod call;
mod inner;

async fn kv_retry_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    let mut times = 0;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        match kv_client(client_poll.clone(), addr.clone()).await {
            Ok(client) => {
                let result = match interface {
                    PlacementCenterInterface::Set => inner_get(client, request.clone()).await,
                    PlacementCenterInterface::Delete => inner_get(client, request.clone()).await,
                    PlacementCenterInterface::Get => inner_get(client, request.clone()).await,
                    PlacementCenterInterface::Exists => inner_get(client, request.clone()).await,
                    _ => return Err(RobustMQError::CommmonError("".to_string())),
                };
                match result {
                    Ok(data) => return Ok(data),
                    Err(e) => {
                        if times > retry_times() {
                            return Err(e);
                        }
                        times = times + 1;
                    }
                }
                sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}


async fn placement_client(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
    let mut poll = client_poll.lock().await;
    match poll.get_placement_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

#[derive(Clone)]
struct KvServiceManager {
    pub addr: String,
}

impl KvServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for KvServiceManager {
    type Connection = KvServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match KvServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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

#[cfg(test)]
mod tests {}
