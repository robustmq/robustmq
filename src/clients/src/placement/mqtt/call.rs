use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService},
    poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        CreateSessionRequest, CreateTopicRequest, CreateUserRequest, DeleteSessionRequest,
        DeleteTopicRequest, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
        ListSessionReply, ListSessionRequest, ListTopicReply, ListTopicRequest, ListUserReply,
        ListUserRequest, SetTopicRetainMessageRequest, UpdateSessionRequest,
    },
};
use std::sync::Arc;

pub async fn placement_get_share_sub_leader(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetShareSubLeaderRequest,
) -> Result<GetShareSubLeaderReply, RobustMQError> {
    let request_data = GetShareSubLeaderRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::GetShareSub,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetShareSubLeaderReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_create_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateUserRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateUser,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteUserRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteUser,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_list_user(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListUserRequest,
) -> Result<ListUserReply, RobustMQError> {
    let request_data = ListUserRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListUser,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListUserReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
pub async fn placement_create_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateTopicRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateTopic,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteTopicRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteTopic,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_list_topic(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListTopicRequest,
) -> Result<ListTopicReply, RobustMQError> {
    let request_data = ListTopicRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListTopic,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListTopicReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_set_topic_retain_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetTopicRetainMessageRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = SetTopicRetainMessageRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::SetTopicRetainMessage,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_create_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: CreateSessionRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = CreateSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::CreateSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_list_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListSessionRequest,
) -> Result<ListSessionReply, RobustMQError> {
    let request_data = ListSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::ListSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListSessionReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_update_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateSessionRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = UpdateSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::UpdateSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_save_last_will_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateSessionRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = UpdateSessionRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::UpdateSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use protocol::placement_center::generate::mqtt::GetShareSubLeaderRequest;

    use crate::placement::mqtt::call::placement_get_share_sub_leader;
    use crate::poll::ClientPool;

    #[tokio::test]
    async fn get_share_sub() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec!["127.0.0.1:1228".to_string()];
        let cluster_name = "test-cluster-name".to_string();
        let group_name = "test-group-name".to_string();
        let request = GetShareSubLeaderRequest {
            group_name,
            cluster_name,
        };
        match placement_get_share_sub_leader(client_poll, addrs, request).await {
            Ok(da) => {
                println!("{:?}", da);
                assert!(true)
            }
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }
    }
}
