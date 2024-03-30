use crate::{placement_center::get_client, retry_times, ClientPool};
use axum::async_trait;
use common_base::errors::RobustMQError;
use protocol::placement_center::placement::{
    placement_center_service_client::PlacementCenterServiceClient, SendRaftConfChangeReply,
    SendRaftConfChangeRequest,
};
use std::{
    marker::{self, PhantomData},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, time::sleep};
use tonic::transport::Channel;

#[async_trait]
pub trait GrpcCall<T> {
    async fn call(&mut self) -> Result<T, RobustMQError>;
}

#[derive(Clone)]
pub struct SendGrpc<T> {
    client: PlacementCenterServiceClient<Channel>,
    message: Vec<u8>,
    _marker: marker::PhantomData<T>,
}

#[async_trait]
impl<T> GrpcCall<T> for SendGrpc<T> {
    async fn call(&mut self) -> Result<T, RobustMQError> {
        let request = SendRaftConfChangeRequest {
            message: self.message.clone(),
        };

        let resp: T = match self
            .client
            .send_raft_conf_change(tonic::Request::new(request))
            .await
        {
            Ok(reply) =>  reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }
}

pub async fn retry_call<T>(mut call: impl GrpcCall<T>) -> Result<T, RobustMQError> {
    let mut times = 0;
    loop {
        match call.call().await {
            Ok(res) => {
                return Ok(res);
            }
            Err(e) => {
                if times > retry_times() {
                    return Err(e);
                }
                times = times + 1;
                sleep(Duration::from_secs(times)).await;
            }
        }
    }
}

pub async fn send_grpc_request(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    message: Vec<u8>,
) -> Result<SendRaftConfChangeReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(client) => {
            let pack = SendGrpc {
                client,
                message,
                _marker: PhantomData::default(),
            };
            return retry_call(pack).await;
        }
        Err(e) => {
            return Err(e);
        }
    }
}
