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
    use protocol::meta::placement_center_kv::kv_service_client::KvServiceClient;
    use protocol::meta::placement_center_kv::{
        DeleteRequest, ExistsRequest, GetRequest, SetRequest,
    };

    use crate::place_server::common::pc_addr;

    #[tokio::test]
    async fn kv_storage() {
        let mut client = KvServiceClient::connect(pc_addr()).await.unwrap();
        let key = "test".to_string();
        let value = "test_value".to_string();
        let set_req = SetRequest {
            key: key.clone(),
            value: value.clone(),
        };
        let _ = client.set(set_req).await;

        let get_req = GetRequest { key: key.clone() };
        let get_rep = client.get(get_req).await.unwrap().into_inner();
        assert_eq!(value, get_rep.value);

        let exists_req = ExistsRequest { key: key.clone() };
        let ex_rep = client.exists(exists_req).await.unwrap().into_inner();
        assert!(ex_rep.flag);

        let del_req = DeleteRequest { key: key.clone() };
        let _ = client.delete(del_req).await.unwrap().into_inner();

        let exists_req = ExistsRequest { key: key.clone() };
        let ex_rep = client.exists(exists_req).await.unwrap().into_inner();
        assert!(!ex_rep.flag);
    }
}
