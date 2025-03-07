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
        mqtt_broker_cluster_status, mqtt_broker_create_schema, mqtt_broker_create_user,
        mqtt_broker_delete_schema, mqtt_broker_delete_user, mqtt_broker_list_schema,
        mqtt_broker_list_user, mqtt_broker_update_schema,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::user::MqttUser;
    use metadata_struct::schema::{SchemaData, SchemaType};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, ListUserRequest,
        MqttCreateSchemaRequest, MqttDeleteSchemaRequest, MqttListSchemaRequest,
        MqttUpdateSchemaRequest,
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

        let create_request = MqttCreateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type: "json".to_string(),
            schema: schema_data.clone(),
            desc: "Old schema".to_string(),
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
                panic!("list schema failed: {}", e);
            }
        }

        // update schema
        schema_data = r#"{
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

        let update_request = MqttUpdateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type: "avro".to_string(),
            schema: schema_data.clone(),
            desc: "New schema".to_string(),
        };

        match mqtt_broker_update_schema(&client_pool, &addrs, update_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("update schema failed: {}", e);
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
                panic!("list schema failed: {}", e);
            }
        }

        // delete schema
        let delete_request = MqttDeleteSchemaRequest {
            schema_name: schema_name.clone(),
        };

        match mqtt_broker_delete_schema(&client_pool, &addrs, delete_request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("delete schema failed: {}", e);
            }
        }

        match mqtt_broker_list_schema(&client_pool, &addrs, list_request).await {
            Ok(reply) => {
                assert_eq!(reply.schemas.len(), 0);
            }

            Err(e) => {
                panic!("list schema failed: {}", e);
            }
        }
    }
}
