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
    use apache_avro::{Schema, Writer};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::sync::Arc;

    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_bind_schema, mqtt_broker_create_schema, mqtt_broker_delete_schema,
        mqtt_broker_unbind_schema,
    };
    use grpc_clients::pool::ClientPool;
    use paho_mqtt::{Message, QOS_1};
    use protocol::broker_mqtt::broker_mqtt_admin::{
        MqttBindSchemaRequest, MqttCreateSchemaRequest, MqttDeleteSchemaRequest,
        MqttUnbindSchemaRequest,
    };

    use crate::mqtt_protocol::common::{
        broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
        publish_data, ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;

    #[tokio::test]
    async fn schema_json_test() {
        let network = "tcp";
        let qos = 1;
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let schema_name = unique_id();
        let schema_type = "json".to_string();
        let schema_content = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        }"#;
        let topic_name = format!("/test/v1/{}", unique_id());

        create_schema(
            client_pool.clone(),
            grpc_addr.clone(),
            schema_name.clone(),
            schema_type.clone(),
            schema_content.to_string(),
            topic_name.clone(),
        )
        .await;

        // Publish
        let client_id = build_client_id(format!("schema_json_test_{}_{}", network, qos).as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let message_content = "mqtt message".to_string();
        let msg = Message::new(topic_name.clone(), message_content, QOS_1);
        publish_data(&cli, msg, true);

        let message_content = json!({"name": "John Doe","age": 30}).to_string();
        println!("message_content:{}", message_content);
        let msg = Message::new(topic_name.clone(), message_content, QOS_1);
        publish_data(&cli, msg, false);

        delete_schema(
            client_pool.clone(),
            grpc_addr.clone(),
            schema_name.clone(),
            topic_name.clone(),
        )
        .await;

        let message_content = "mqtt message".to_string();
        let msg = Message::new(topic_name.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, false);
        distinct_conn(cli);
    }

    #[tokio::test]
    async fn schema_avro_test() {
        let network: &str = "tcp";
        let qos = 1;
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let schema_name = unique_id();
        let schema_type = "avro".to_string();
        let schema_content = r#"
        {
            "type": "record",
            "name": "test",
            "fields": [
                {"name": "a", "type": "long"},
                {"name": "b", "type": "string"}
            ]
        }
        "#;

        let topic_name = format!("/test/v1/{}", unique_id());

        create_schema(
            client_pool.clone(),
            grpc_addr.clone(),
            schema_name.clone(),
            schema_type.clone(),
            schema_content.to_string(),
            topic_name.clone(),
        )
        .await;

        // Publish
        let client_id = build_client_id(format!("schema_avro_test_{}_{}", network, qos).as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // fail
        let message_content = "mqtt message".to_string();
        let msg = Message::new(topic_name.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, true);

        // success
        let schema = Schema::parse_str(schema_content).unwrap();
        let test_data = TestData {
            a: 1,
            b: "test".to_string(),
        };
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append_ser(test_data).unwrap();
        let encoded_data = writer.into_inner().unwrap();
        println!("encoded_data len: {:?}", encoded_data.len());
        let msg = Message::new(topic_name.clone(), encoded_data, QOS_1);
        publish_data(&cli, msg, false);

        delete_schema(
            client_pool.clone(),
            grpc_addr.clone(),
            schema_name.clone(),
            topic_name.clone(),
        )
        .await;

        let message_content = "mqtt message".to_string();
        let msg = Message::new(topic_name.clone(), message_content.clone(), QOS_1);
        publish_data(&cli, msg, false);
        distinct_conn(cli);
    }

    async fn create_schema(
        client_pool: Arc<ClientPool>,
        addrs: Vec<String>,
        schema_name: String,
        schema_type: String,
        schema: String,
        topic_name: String,
    ) {
        let user = MqttCreateSchemaRequest {
            schema_name: schema_name.clone(),
            schema_type,
            schema,
            desc: "".to_string(),
        };
        let res = mqtt_broker_create_schema(&client_pool, &addrs, user.clone()).await;
        assert!(res.is_ok());

        let request = MqttBindSchemaRequest {
            schema_name: schema_name.clone(),
            resource_name: topic_name,
        };
        let res = mqtt_broker_bind_schema(&client_pool, &addrs, request).await;
        assert!(res.is_ok());
    }

    async fn delete_schema(
        client_pool: Arc<ClientPool>,
        addrs: Vec<String>,
        schema_name: String,
        topic_name: String,
    ) {
        let user = MqttDeleteSchemaRequest {
            schema_name: schema_name.clone(),
        };
        let res = mqtt_broker_delete_schema(&client_pool, &addrs, user.clone()).await;
        assert!(res.is_ok());

        let request = MqttUnbindSchemaRequest {
            schema_name: schema_name.clone(),
            resource_name: topic_name,
        };
        let res = mqtt_broker_unbind_schema(&client_pool, &addrs, request).await;
        assert!(res.is_ok());
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestData {
        a: u64,
        b: String,
    }
}
