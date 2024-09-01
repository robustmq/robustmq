mod common;

#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{
        placement::kv::call::{placement_delete, placement_exists, placement_get, placement_set},
        poll::ClientPool,
    };
    use protocol::placement_center::generate::kv::{
        DeleteRequest, ExistsRequest, GetRequest, SetRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn kv_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];
        let key = "test-sub-name".to_string();
        let value = "test-group-name".to_string();
        let request = SetRequest {
            key: key.clone(),
            value: value.clone(),
        };
        match placement_set(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        let exist_req = ExistsRequest { key: key.clone() };
        match placement_exists(client_poll.clone(), addrs.clone(), exist_req).await {
            Ok(da) => {
                assert!(da.flag)
            }
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        let get_req = GetRequest { key: key.clone() };
        match placement_get(client_poll.clone(), addrs.clone(), get_req).await {
            Ok(da) => {
                assert_eq!(da.value, value);
            }
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        let exist_req = DeleteRequest { key: key.clone() };
        match placement_delete(client_poll.clone(), addrs.clone(), exist_req).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }

        let exist_req = ExistsRequest { key: key.clone() };
        match placement_exists(client_poll.clone(), addrs.clone(), exist_req).await {
            Ok(da) => {
                assert!(!da.flag)
            }
            Err(e) => {
                println!("{}", e.to_string());
                assert!(false)
            }
        }
    }
}
