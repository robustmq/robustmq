use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService},
    poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{DeleteShareSubRequest, GetShareSubReply, GetShareSubRequest},
};
use std::sync::Arc;

pub async fn placement_get_share_sub(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: GetShareSubRequest,
) -> Result<GetShareSubReply, RobustMQError> {
    let request_data = GetShareSubRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::GetShareSub,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match GetShareSubReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_delete_share_sub(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteShareSubRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = DeleteShareSubRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::Mqtt,
        PlacementCenterInterface::DeleteShareSub,
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

    use protocol::placement_center::generate::mqtt::GetShareSubRequest;

    use crate::placement::mqtt::call::placement_get_share_sub;
    use crate::poll::ClientPool;

    #[tokio::test]
    async fn get_share_sub() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec!["127.0.0.1:1228".to_string()];
        let cluster_name = "test-cluster-name".to_string();
        let sub_name = "test-sub-name".to_string();
        let group_name = "test-group-name".to_string();
        let request = GetShareSubRequest {
            group_name,
            sub_name,
            cluster_name,
        };
        match placement_get_share_sub(client_poll, addrs, request).await {
            Ok(da) => {}
            Err(e) => {}
        }
    }
}
