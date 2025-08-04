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
        mqtt_broker_create_user, mqtt_broker_delete_connector, mqtt_broker_delete_schema,
        mqtt_broker_delete_user, mqtt_broker_list_connector, mqtt_broker_list_schema,
        mqtt_broker_list_user, mqtt_broker_update_connector, mqtt_broker_update_schema,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::bridge::config_kafka::KafkaConnectorConfig;
    use metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig;
    use metadata_struct::mqtt::bridge::connector_type::ConnectorType;
    use metadata_struct::schema::{SchemaData, SchemaType};
    use prost::Message;
    use protocol::broker_mqtt::broker_mqtt_admin::{self, ConnectorRaw};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        ClusterStatusRequest, CreateConnectorRequest, CreateSchemaRequest, CreateUserRequest,
        DeleteConnectorRequest, DeleteSchemaRequest, DeleteUserRequest, ListConnectorRequest,
        ListSchemaRequest, ListUserRequest, UpdateConnectorRequest, UpdateSchemaRequest,
    };

    use crate::common::get_mqtt_broker_addr;

    #[tokio::test]
    async fn cluster_status_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let request = ClusterStatusRequest {};
        match mqtt_broker_cluster_status(&client_pool, &addrs, request).await {
            Ok(data) => {
                println!("{data:?}");
            }
            Err(e) => {
                panic!("{e:?}");
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
                panic!("{e:?}");
            }
        }

        match mqtt_broker_list_user(&client_pool, &addrs, ListUserRequest { options: None }).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    if user.username == raw.username {
                        flag = true;
                    }
                }
                assert!(flag, "user1 has been created");
            }
            Err(e) => {
                panic!("{e:?}");
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
                panic!("{e:?}");
            }
        }

        match mqtt_broker_list_user(&client_pool, &addrs, ListUserRequest { options: None }).await {
            Ok(data) => {
                let mut flag = true;
                for raw in data.users {
                    if user.username == raw.username {
                        flag = false;
                    }
                }
                assert!(flag, "user1 should be deleted");
            }
            Err(e) => {
                panic!("{e:?}");
            }
        };
    }

    #[tokio::test]
    async fn schema_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        let schema_name = "test_schema".to_string();

        let mut schema_data = r#"{
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

        let create_request = CreateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type: "json".to_string(),
            schema: schema_data.clone(),
            desc: "Old schema".to_string(),
        };

        match mqtt_broker_create_schema(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("create schema failed: {e}");
            }
        }

        let list_request = ListSchemaRequest {
            schema_name: schema_name.clone(),
        };

        match mqtt_broker_list_schema(&client_pool, &addrs, list_request.clone()).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema =
                    serde_json::from_slice::<SchemaData>(reply.schemas.first().unwrap()).unwrap();

                assert_eq!(schema.name, schema_name);
                assert_eq!(schema.schema_type, SchemaType::JSON);
                assert_eq!(schema.schema, schema_data);
                assert_eq!(schema.desc, "Old schema".to_string());
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }

        // update schema
        schema_data = r#"{
            "type": "record",
            "name": "test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"}
            ]
        }"#
        .to_string();

        let update_request = UpdateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type: "avro".to_string(),
            schema: schema_data.clone(),
            desc: "New schema".to_string(),
        };

        match mqtt_broker_update_schema(&client_pool, &addrs, update_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("update schema failed: {e}");
            }
        }

        // list the request just updated
        match mqtt_broker_list_schema(&client_pool, &addrs, list_request.clone()).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 1);
                let schema =
                    serde_json::from_slice::<SchemaData>(reply.schemas.first().unwrap()).unwrap();

                assert_eq!(schema.name, schema_name);
                assert_eq!(schema.schema_type, SchemaType::AVRO);
                assert_eq!(schema.schema, schema_data);
                assert_eq!(schema.desc, "New schema".to_string());
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }

        // delete schema
        let delete_request = DeleteSchemaRequest {
            schema_name: schema_name.clone(),
        };

        match mqtt_broker_delete_schema(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("delete schema failed: {e}");
            }
        }

        match mqtt_broker_list_schema(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 0);
            }

            Err(e) => {
                panic!("list schema failed: {e}");
            }
        }
    }

    #[tokio::test]
    async fn connector_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_mqtt_broker_addr()];

        // create connector
        let connector_name = "test_connector-1".to_string();

        let create_request = CreateConnectorRequest {
            connector_name: connector_name.clone(),
            connector_type: broker_mqtt_admin::ConnectorType::File as i32,
            config: serde_json::to_string(&LocalFileConnectorConfig {
                local_file_path: "/tmp/test".to_string(),
            })
            .unwrap(),
            topic_id: "test-topic-1".to_string(),
        };

        match mqtt_broker_create_connector(&client_pool, &addrs, create_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        // list connector we just created
        let list_request = ListConnectorRequest {
            connector_name: connector_name.clone(),
            options: None,
        };

        let mut connector: ConnectorRaw =
            match mqtt_broker_list_connector(&client_pool, &addrs, list_request.clone()).await {
                Ok(reply) => {
                    assert_eq!(reply.connectors.len(), 1);
                    reply.connectors.first().unwrap().clone()
                }

                Err(e) => {
                    panic!("{e:?}");
                }
            };

        assert_eq!(&connector.connector_name, &connector_name);
        // assert_eq!(connector.connector_type.clone(), ConnectorType::LocalFile);
        assert_eq!(connector.connector_type, ConnectorType::LocalFile as i32);
        assert_eq!(
            &connector.config,
            &serde_json::to_string(&LocalFileConnectorConfig {
                local_file_path: "/tmp/test".to_string(),
            })
            .unwrap()
        );
        assert_eq!(&connector.topic_id, "test-topic-1");

        // update
        connector.connector_type = ConnectorType::Kafka as i32;
        connector.config = serde_json::to_string(&KafkaConnectorConfig {
            bootstrap_servers: "127.0.0.1:9092".to_string(),
            topic: "test-topic".to_string(),
            key: "test-key".to_string(),
        })
        .unwrap();
        connector.topic_id = "test-topic-2".to_string();

        let update_request = UpdateConnectorRequest {
            // connector: serde_json::to_vec(&connector).unwrap(),
            connector: connector.encode_to_vec(),
        };

        match mqtt_broker_update_connector(&client_pool, &addrs, update_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        // list connector we just updated
        connector =
            match mqtt_broker_list_connector(&client_pool, &addrs, list_request.clone()).await {
                Ok(reply) => {
                    assert_eq!(reply.connectors.len(), 1);
                    reply.connectors.first().unwrap().clone()
                }

                Err(e) => {
                    panic!("{e:?}");
                }
            };

        assert_eq!(&connector.connector_name, &connector_name);
        assert_eq!(
            connector.connector_type.clone(),
            ConnectorType::Kafka as i32
        );

        assert_eq!(
            &connector.config,
            &serde_json::to_string(&KafkaConnectorConfig {
                bootstrap_servers: "127.0.0.1:9092".to_string(),
                topic: "test-topic".to_string(),
                key: "test-key".to_string(),
            })
            .unwrap()
        );
        assert_eq!(&connector.topic_id, "test-topic-2");

        // delete connector
        let delete_request = DeleteConnectorRequest {
            connector_name: connector_name.clone(),
        };

        match mqtt_broker_delete_connector(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        }

        // list connector we just deleted
        match mqtt_broker_list_connector(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.connectors.len(), 0);
            }

            Err(e) => {
                panic!("{e:?}");
            }
        }
    }
}
