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
mod test {
    use std::sync::Arc;

    use grpc_clients::{
        placement::mqtt::call::{placement_create_connector, placement_list_connector},
        pool::ClientPool,
    };
    use metadata_struct::mqtt::bridge::connector::MQTTConnector;
    use protocol::placement_center::placement_center_mqtt::{
        CreateConnectorRequest, ListConnectorRequest,
    };

    use crate::common::get_placement_addr;

    fn check_connector_equal(left: &MQTTConnector, right: &MQTTConnector) {
        assert_eq!(left.cluster_name, right.cluster_name);
        assert_eq!(left.connector_name, right.connector_name);
        assert_eq!(
            left.connector_type.clone() as u8,
            right.connector_type.clone() as u8
        );
        assert_eq!(left.config, right.config);
        assert_eq!(left.topic_id, right.topic_id);
        assert_eq!(left.status.clone() as u8, right.status.clone() as u8);
        assert_eq!(left.broker_id, right.broker_id);
    }

    #[tokio::test]
    async fn connector_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];

        let cluster_name: String = "test_cluster".to_string();
        let connector_name = "test_connector".to_string();

        // create connector
        let connector = MQTTConnector {
            cluster_name: cluster_name.clone(),
            connector_name: connector_name.clone(),
            ..Default::default()
        };

        let create_request = CreateConnectorRequest {
            cluster_name: cluster_name.clone(),
            connector_name: connector_name.clone(),
            connector: serde_json::to_vec(&connector).unwrap(),
        };

        match placement_create_connector(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // list the connector we just created
        let list_request = ListConnectorRequest {
            cluster_name: cluster_name.clone(),
            connector_name: connector_name.clone(),
        };

        match placement_list_connector(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.connectors.len(), 1);

                for connector_bytes in reply.connectors {
                    let mqtt_connector =
                        serde_json::from_slice::<MQTTConnector>(&connector_bytes).unwrap();

                    check_connector_equal(&mqtt_connector, &connector);
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
