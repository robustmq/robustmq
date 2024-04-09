use super::manager::kv_client;
use crate::{retry_sleep_time, retry_times, ClientPool};
use common_base::{errors::RobustMQError, log::error};
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

pub async fn placement_set(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: SetRequest,
) -> Result<CommonReply, RobustMQError> {
    match kv_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client.set(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error(format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "placement_set"
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

pub async fn placement_get(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: GetRequest,
) -> Result<GetReply, RobustMQError> {
    match kv_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client.get(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error(format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "placement_get"
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

pub async fn placement_delete(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: DeleteRequest,
) -> Result<CommonReply, RobustMQError> {
    match kv_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client.delete(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error(format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "placement_delete"
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

pub async fn placement_exists(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: ExistsRequest,
) -> Result<ExistsReply, RobustMQError> {
    match kv_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client.exists(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error(format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr,
                            "placement_exists"
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
