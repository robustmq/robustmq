use common::errors::RobustMQError;
use protocol::placement_center::placement::{
    placement_center_service_client::PlacementCenterServiceClient, CommonReply, CreateShardRequest,
    DeleteShardRequest, HeartbeatRequest, RegisterNodeRequest, SendRaftConfChangeReply,
    SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest, UnRegisterNodeRequest,
};
use std::collections::HashMap;
use tonic::transport::Channel;

#[derive(Debug,Clone)]
pub struct PlacementCenterClientManager {
    clients: HashMap<String, PlacementCenterServiceClient<Channel>>,
}

impl PlacementCenterClientManager {
    pub fn new() -> Self {
        let clients = HashMap::new();
        return PlacementCenterClientManager { clients };
    }

    // Try to obtain an available HTTP connection
    pub async fn placement_center_client(
        &mut self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        if let Some(client) = self.clients.remove(&addr) {
            self.clients.insert(addr, client.clone());
            return Ok(client);
        }

        match PlacementCenterServiceClient::connect(format!("http://{}", addr.clone())).await {
            Ok(client) => {
                self.clients.insert(addr.clone(), client.clone());
                return Ok(client);
            }
            Err(err) => return Err(RobustMQError::TonicTransport(err)),
        };
    }

    pub async fn register_node(
        &mut self,
        addr: String,
        request: RegisterNodeRequest,
    ) -> Result<CommonReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let resp = match client.register_node(tonic::Request::new(request)).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn unregister_node(
        &mut self,
        addr: String,
        request: UnRegisterNodeRequest,
    ) -> Result<CommonReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let resp = match client.un_register_node(tonic::Request::new(request)).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn create_shard(
        &mut self,
        addr: String,
        request: CreateShardRequest,
    ) -> Result<CommonReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let resp = match client.create_shard(tonic::Request::new(request)).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn delete_shard(
        &mut self,
        addr: String,
        request: DeleteShardRequest,
    ) -> Result<CommonReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let resp = match client.delete_shard(tonic::Request::new(request)).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn heartbeat(
        &mut self,
        addr: String,
        request: HeartbeatRequest,
    ) -> Result<CommonReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let resp = match client.heartbeat(tonic::Request::new(request)).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn send_raft_message(
        &mut self,
        addr: String,
        message: Vec<u8>,
    ) -> Result<SendRaftMessageReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let request = tonic::Request::new(SendRaftMessageRequest { message });

        let resp = match client.send_raft_message(request).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }

    pub async fn send_raft_conf_change(
        &mut self,
        addr: String,
        message: Vec<u8>,
    ) -> Result<SendRaftConfChangeReply, RobustMQError> {
        let mut client = match self.placement_center_client(addr).await {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let request = tonic::Request::new(SendRaftConfChangeRequest { message });

        let resp = match client.send_raft_conf_change(request).await {
            Ok(reply) => reply.into_inner(),
            Err(status) => return Err(RobustMQError::MetaGrpcStatus(status)),
        };
        return Ok(resp);
    }
}
