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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use grpc_clients::{
        meta::common::call::{kv_delete, kv_exists, kv_get, kv_set},
        pool::ClientPool,
    };
    use protocol::meta::meta_service_common::{
        DeleteRequest, ExistsRequest, GetRequest, SetRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]

    async fn kv_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];
        let key = "test-sub-name".to_string();
        let value = "test-group-name".to_string();
        let request = SetRequest {
            key: key.clone(),
            value: value.clone(),
        };
        match kv_set(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let request_key_empty = SetRequest {
            key: "".to_string(),
            value: value.clone(),
        };
        let err = kv_set(&client_pool, &addrs, request_key_empty)
            .await
            .unwrap_err();
        println!("{:?}", err);
        assert!(err
            .to_string()
            .contains("characters length must be greater than or equal to 1"));

        let request_value_empty = SetRequest {
            key: key.clone(),
            value: "".to_string(),
        };
        let err = kv_set(&client_pool, &addrs, request_value_empty)
            .await
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("characters length must be greater than or equal to 1"));

        let exist_req = ExistsRequest { key: key.clone() };
        match kv_exists(&client_pool, &addrs, exist_req).await {
            Ok(da) => {
                assert!(da.flag)
            }
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let get_req = GetRequest { key: key.clone() };
        match kv_get(&client_pool, &addrs, get_req).await {
            Ok(da) => {
                assert_eq!(da.value, value);
            }
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let exist_req = DeleteRequest { key: key.clone() };
        match kv_delete(&client_pool, &addrs, exist_req).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        let exist_req: ExistsRequest = ExistsRequest { key: key.clone() };
        match kv_exists(&client_pool, &addrs, exist_req).await {
            Ok(da) => {
                assert!(!da.flag)
            }
            Err(e) => {
                panic!("{e:?}");
            }
        }
    }
}
