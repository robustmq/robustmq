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

use crate::mqtt_protocol::common::broker_grpc_addr;
use common_base::enum_type::feature_type::FeatureType;
use grpc_clients::mqtt::admin::call::mqtt_broker_set_cluster_config;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::{SetClusterConfigReply, SetClusterConfigRequest};
use std::sync::Arc;

#[tokio::test]
async fn test_enable_offline_message() {
    let client_pool = Arc::new(ClientPool::new(3));
    let grpc_addr = vec![broker_grpc_addr()];

    let is_enables = [true, false];
    for is_enable in is_enables.iter() {
        let request = SetClusterConfigRequest {
            feature_name: FeatureType::OfflineMessage.to_string(),
            is_enable: *is_enable,
        };

        let reply = SetClusterConfigReply {
            feature_name: FeatureType::OfflineMessage.to_string(),
            is_enable: *is_enable,
        };

        match mqtt_broker_set_cluster_config(&client_pool, &grpc_addr, request).await {
            Ok(data) => {
                assert_eq!(reply, data);
            }

            Err(e) => {
                eprintln!("Failed enable_offline_message: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
