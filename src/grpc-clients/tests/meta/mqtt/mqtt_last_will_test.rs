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

    use common_base::tools::unique_id;
    use grpc_clients::meta::mqtt::call::placement_save_last_will_message;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::lastwill::MqttLastWillData;
    use protocol::meta::meta_service_mqtt::SaveLastWillMessageRequest;

    use crate::common::get_placement_addr;

    #[tokio::test]

    async fn mqtt_last_will_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = unique_id();
        let client_id: String = unique_id();

        let last_will_message = MqttLastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: None,
        };

        let request = SaveLastWillMessageRequest {
            cluster_name: cluster_name.clone(),
            client_id: client_id.clone(),
            last_will_message: last_will_message.encode().unwrap(),
        };
        match placement_save_last_will_message(&client_pool, &addrs, request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }
    }
}
