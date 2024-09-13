#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{
        placement::mqtt::call::placement_save_last_will_message,
        poll::ClientPool,
    };
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::lastwill::LastWillData;
    use protocol::placement_center::generate::mqtt::SaveLastWillMessageRequest;
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_last_will_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = unique_id();
        let client_id: String = unique_id();

        let last_will_message = LastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: None,
        };

        let request = SaveLastWillMessageRequest {
            cluster_name: cluster_name.clone(),
            client_id: client_id.clone(),
            last_will_message: last_will_message.encode(),
        };
        match placement_save_last_will_message(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}