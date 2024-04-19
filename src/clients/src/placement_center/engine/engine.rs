use std::{sync::Arc, time::Duration};
use common_base::{errors::RobustMQError, log::error_meta};
use mobc::Manager;
use protocol::placement_center::generate::{
    common::CommonReply,
    engine::{engine_service_client::EngineServiceClient, CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest},
};
use tokio::{sync::Mutex, time::sleep};
use tonic::transport::Channel;
use crate::{retry_sleep_time, retry_times, ClientPool};
use super::manager::engine_client;

pub async fn create_shard(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: CreateShardRequest,
) -> Result<CommonReply, RobustMQError> {
    match engine_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .create_shard(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error_meta(&format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "create_shard"
                        ));
                        if times > retry_times() {
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(retry_sleep_time(times))).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn delete_shard(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: DeleteShardRequest,
) -> Result<CommonReply, RobustMQError> {
    match engine_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .delete_shard(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error_meta(&format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "delete_shard"
                        ));
                        if times > retry_times() {
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(retry_sleep_time(times))).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn create_segment(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: CreateSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    match engine_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .create_segment(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error_meta(&format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "create_segment"
                        ));
                        if times > retry_times() {
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(retry_sleep_time(times))).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn delete_segment(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: DeleteSegmentRequest,
) -> Result<CommonReply, RobustMQError> {
    match engine_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .delete_segment(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error_meta(&format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "delete_segment"
                        ));
                        if times > retry_times() {
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(retry_sleep_time(times))).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

#[derive(Clone)]
pub struct EngineServiceManager {
    pub addr: String,
}

impl EngineServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for EngineServiceManager {
    type Connection = EngineServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match EngineServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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

