mod common;
#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{placement::mqtt::call::list_blacklist, poll::ClientPool};
    use protocol::placement_center::generate::mqtt::{CreateBlacklistRequest, ListBlacklistRequest};
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_blacklist_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = "test".to_string();
        let blacklist = MQTTAclBl{

        };
        let request = CreateBlacklistRequest {
            cluster_name: cluster_name.clone(),
            blacklist: blacklist.en
        };

        let request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };
        match list_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}
