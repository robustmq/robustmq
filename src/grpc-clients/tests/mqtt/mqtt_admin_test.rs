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

    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_cluster_status, mqtt_broker_create_connector, mqtt_broker_create_schema,
        mqtt_broker_create_user, mqtt_broker_delete_connector, mqtt_broker_delete_user,
        mqtt_broker_list_connector, mqtt_broker_list_schema, mqtt_broker_list_user,
        mqtt_broker_update_connector,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig;
    use metadata_struct::mqtt::bridge::connector::MQTTConnector;
    use metadata_struct::mqtt::bridge::connector_type::ConnectorType;
    use metadata_struct::mqtt::user::MqttUser;
    use metadata_struct::schema::{SchemaData, SchemaType};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, ListUserRequest,
        MqttCreateConnectorRequest, MqttCreateSchemaRequest, MqttDeleteConnectorRequest,
        MqttListConnectorRequest, MqttListSchemaRequest, MqttUpdateConnectorRequest,
    };

    use crate::common::get_mqtt_broker_addr;

    #[tokio::test]
    async fn cluster_status_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let request = ClusterStatusRequest {};
        match mqtt_broker_cluster_status(&client_pool, &addrs, request).await {
            Ok(data) => {
                println!("{:?}", data);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn user_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];
        let user_name: String = "user1".to_string();
        let password: String = "123456".to_string();

        let user = CreateUserRequest {
            username: user_name.clone(),
            password: password.clone(),
            is_superuser: false,
        };

        match mqtt_broker_create_user(&client_pool, &addrs, user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        match mqtt_broker_list_user(&client_pool, &addrs, ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = true;
                    }
                }
                assert!(flag, "user1 has been created");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        match mqtt_broker_delete_user(
            &client_pool,
            &addrs,
            DeleteUserRequest {
                username: user.username.clone(),
            },
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        match mqtt_broker_list_user(&client_pool, &addrs, ListUserRequest {}).await {
            Ok(data) => {
                let mut flag = true;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = false;
                    }
                }
                assert!(flag, "user1 should be deleted");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        };
    }

    #[tokio::test]
    async fn schema_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let schema_name = "test_schema".to_string();

        let schema_data = r#"{
                "type":"object",
                "properties":{
                    "name":{
                        "type": "string"
                    },
                    "age":{
                        "type": "integer", "minimum": 0
                    }
                },
                "required":["name"]
            }"#
        .to_string();

        let create_request = MqttCreateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type: "json".to_string(),
            schema: schema_data.clone(),
            desc: "".to_string(),
        };

        match mqtt_broker_create_schema(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("create schema failed: {}", e);
            }
        }

        let list_request = MqttListSchemaRequest {
            schema_name: schema_name.clone(),
        };

        match mqtt_broker_list_schema(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema =
                    serde_json::from_slice::<SchemaData>(reply.schemas.first().unwrap()).unwrap();

                assert_eq!(schema.name, schema_name);
                assert_eq!(schema.schema_type, SchemaType::JSON);
                assert_eq!(schema.schema, schema_data);
                assert_eq!(schema.desc, "".to_string());
            }

            Err(e) => {
                panic!("list schema failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn connector_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        // create connector
        let connector_name = "test_connector".to_string();

        let config = serde_json::to_string(&LocalFileConnectorConfig {
            local_file_path: "/tmp/test".to_string(),
        })
        .unwrap();

        let create_request = MqttCreateConnectorRequest {
            connector_name: connector_name.clone(),
            connector_type: ConnectorType::Kafka as i32,
            config: config.clone(),
            topic_id: "test-topic".to_string(),
        };

        match mqtt_broker_create_connector(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // list connector we just created
        let list_request = MqttListConnectorRequest {
            connector_name: connector_name.clone(),
        };

        let mut connector =
            match mqtt_broker_list_connector(&client_pool, &addrs, list_request.clone()).await {
                Ok(reply) => {
                    assert_eq!(reply.connectors.len(), 1);
                    serde_json::from_slice::<MQTTConnector>(reply.connectors.first().unwrap())
                        .unwrap()
                }

                Err(e) => {
                    panic!("{:?}", e);
                }
            };

        assert_eq!(&connector.connector_name, &connector_name);
        assert_eq!(
            connector.connector_type.clone() as i32,
            ConnectorType::Kafka as i32
        );
        assert_eq!(&connector.config, &config);
        assert_eq!(&connector.topic_id, "test-topic");

        // update
        let new_config = serde_json::to_string(&LocalFileConnectorConfig {
            local_file_path: "/tmp/test2".to_string(),
        })
        .unwrap();

        connector.config = new_config.clone();
        connector.connector_type = ConnectorType::LocalFile;

        let update_request = MqttUpdateConnectorRequest {
            connector: serde_json::to_vec(&connector).unwrap(),
        };

        match mqtt_broker_update_connector(&client_pool, &addrs, update_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // list connector we just updated
        connector = match mqtt_broker_list_connector(&client_pool, &addrs, list_request.clone())
            .await
        {
            Ok(reply) => {
                assert_eq!(reply.connectors.len(), 1);
                serde_json::from_slice::<MQTTConnector>(reply.connectors.first().unwrap()).unwrap()
            }

            Err(e) => {
                panic!("{:?}", e);
            }
        };

        assert_eq!(&connector.connector_name, &connector_name);
        assert_eq!(
            connector.connector_type.clone() as i32,
            ConnectorType::LocalFile as i32
        );

        assert_eq!(&connector.config, &new_config);
        assert_eq!(&connector.topic_id, "test-topic");

        // delete connector
        let delete_request = MqttDeleteConnectorRequest {
            connector_name: connector_name.clone(),
        };

        match mqtt_broker_delete_connector(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // list connector we just deleted
        match mqtt_broker_list_connector(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.connectors.len(), 0);
            }

            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
