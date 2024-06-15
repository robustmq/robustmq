use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService},
    poll::ClientPool,
};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest},
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
