// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

        let request_key_empty = SetRequest {
            key: "".to_string(),
            value: value.clone(),
        };
        match placement_set(client_poll.clone(), addrs.clone(), request_key_empty).await{
            Ok(_) => {
                panic!("Expected an error when key is empty, but got ok");
            }
            Err(e) => {
                assert!(e.to_string().contains("key or value"));
            }
        }

        let request_value_empty = SetRequest{
            key: "test-sub-name".to_string(),
            value: "".to_string(),
        };
        match placement_set(client_poll.clone(), addrs.clone(), request_value_empty).await{
            Ok(_) => {
                panic!("Expected an error when value is empty, but got ok");
            }
            Err(e) => {
                assert!(e.to_string().contains("key or value"));
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
