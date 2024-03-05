/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::{retry_times, ClientPool};
use common::{errors::RobustMQError, log::error_meta};
use mobc::Manager;
use protocol::placement_center::placement::{
    placement_center_service_client::PlacementCenterServiceClient, CommonReply, CreateShardRequest,
    DeleteShardRequest, HeartbeatRequest, RegisterNodeRequest, SendRaftConfChangeReply,
    SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest, UnRegisterNodeRequest,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use tonic::transport::Channel;

pub struct PlacementCenterConnectionManager {
    pub addr: String,
}

impl PlacementCenterConnectionManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for PlacementCenterConnectionManager {
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

pub async fn register_node(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: RegisterNodeRequest,
) -> Result<CommonReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .register_node(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn unregister_node(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: UnRegisterNodeRequest,
) -> Result<CommonReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .un_register_node(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn create_shard(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: CreateShardRequest,
) -> Result<CommonReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .create_shard(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
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
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client
                    .delete_shard(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn heartbeat(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    request: HeartbeatRequest,
) -> Result<CommonReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let mut times = 0;
            loop {
                match client.heartbeat(tonic::Request::new(request.clone())).await {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn send_raft_message(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    message: Vec<u8>,
) -> Result<SendRaftMessageReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let request = SendRaftMessageRequest { message };
            let mut times = 0;
            loop {
                match client
                    .send_raft_message(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn send_raft_conf_change(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
    message: Vec<u8>,
) -> Result<SendRaftConfChangeReply, RobustMQError> {
    match get_client(client_poll, addr.clone()).await {
        Ok(mut client) => {
            let request = SendRaftConfChangeRequest { message };
            let mut times = 0;
            loop {
                match client
                    .send_raft_conf_change(tonic::Request::new(request.clone()))
                    .await
                {
                    Ok(reply) => return Ok(reply.into_inner()),
                    Err(status) => {
                        if times > retry_times() {
                            error_meta("");
                            return Err(RobustMQError::MetaGrpcStatus(status));
                        }
                        times = times + 1;
                        sleep(Duration::from_secs(times)).await;
                    }
                };
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn get_client(
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
    let mut poll = client_poll.lock().await;
    match poll.get_placement_center_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}
