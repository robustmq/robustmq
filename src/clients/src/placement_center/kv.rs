use super::manager::kv_client;
use crate::{retry_sleep_time, retry_times, ClientPool};
use common_base::{errors::RobustMQError, log::error};
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use tonic::IntoRequest;

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

pub struct Resp<T> {
    data: T,
}

pub async fn placement_kv_request<T, U>(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: T,
) -> Result<U, RobustMQError>
where
    T: Clone,
    tonic::Request<T>: IntoRequest<GetRequest>,
{
    let mut times = 0;
    let mut error_status;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        match kv_client(client_poll.clone(), addr.clone()).await {
            Ok(mut client) => {
                match client.get(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error_status = status;
                    }
                };

                if times > retry_times() {
                    error(format!(
                        "{},target ip:{},call function:{}",
                        error_status.to_string(),
                        addr.clone(),
                        "placement_get"
                    ));
                    return Err(RobustMQError::MetaGrpcStatus(error_status));
                }
                times = times + 1;
                sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub async fn placement_get(
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: GetRequest,
) -> Result<GetReply, RobustMQError> {
    let mut times = 0;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        match kv_client(client_poll.clone(), addr.clone()).await {
            Ok(mut client) => {
                match client.get(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        error(format!(
                            "{},target ip:{},call function:{}",
                            status.to_string(),
                            addr.clone(),
                            "placement_get"
                        ));
                        if times > retry_times() {
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
                    }
                };
            }
            Err(e) => {
                return Err(e);
            }
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
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use protocol::placement_center::generate::kv::{GetReply, GetRequest, SetRequest};
    use tokio::sync::Mutex;

    use crate::{placement_center::kv::placement_kv_request, ClientPool};
    #[tokio::test]
    async fn placement_kv_request_set() {
        let client_poll: Arc<Mutex<ClientPool>> = Arc::new(Mutex::new(ClientPool::new(1)));
        let addrs = vec!["14.103.42.35:1228".to_string()];
        let get = GetRequest {
            key: "name".to_string(),
        };
        placement_kv_request::<GetRequest, GetReply>(client_poll, addrs, get).await;
    }
}
