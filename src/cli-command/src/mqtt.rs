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

use crate::template::{PublishArgsRequest, SubscribeArgsRequest};
use crate::{connect_server5, error_info, grpc_addr};
use common_base::enum_type::sort_type::SortType;
use common_base::tools::unique_id;
use grpc_clients::mqtt::admin::call::{
    mqtt_broker_bind_schema, mqtt_broker_cluster_status, mqtt_broker_create_connector,
    mqtt_broker_create_schema, mqtt_broker_create_user, mqtt_broker_delete_connector,
    mqtt_broker_delete_schema, mqtt_broker_delete_user, mqtt_broker_enable_flapping_detect,
    mqtt_broker_enable_slow_subscribe, mqtt_broker_list_bind_schema, mqtt_broker_list_connection,
    mqtt_broker_list_connector, mqtt_broker_list_schema, mqtt_broker_list_slow_subscribe,
    mqtt_broker_list_topic, mqtt_broker_list_user, mqtt_broker_unbind_schema,
    mqtt_broker_update_connector, mqtt_broker_update_schema,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::schema::SchemaData;
use paho_mqtt::{DisconnectOptionsBuilder, MessageBuilder, Properties, PropertyCode, ReasonCode};
use prettytable::{row, Table};
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, EnableFlappingDetectRequest,
    EnableSlowSubscribeRequest, ListConnectionRequest, ListSlowSubscribeRequest, ListTopicRequest,
    ListUserRequest, MqttBindSchemaRequest, MqttCreateConnectorRequest, MqttCreateSchemaRequest,
    MqttDeleteConnectorRequest, MqttDeleteSchemaRequest, MqttListBindSchemaRequest,
    MqttListConnectorRequest, MqttListSchemaRequest, MqttUnbindSchemaRequest,
    MqttUpdateConnectorRequest, MqttUpdateSchemaRequest,
};
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::{select, signal};

#[derive(Clone)]
pub struct MqttCliCommandParam {
    pub server: String,
    pub action: MqttActionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MqttActionType {
    Status,

    // User admin
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),
    ListUser,

    // connection
    ListConnection,

    // observability: slow-sub
    EnableSlowSubscribe(EnableSlowSubscribeRequest),
    ListSlowSubscribe(ListSlowSubscribeRequest),

    // flapping detect
    EnableFlappingDetect(EnableFlappingDetectRequest),

    // publish
    Publish(PublishArgsRequest),

    // subscribe
    Subscribe(SubscribeArgsRequest),

    ListTopic(ListTopicRequest),

    // connector
    ListConnector(MqttListConnectorRequest),
    CreateConnector(MqttCreateConnectorRequest),
    UpdateConnector(MqttUpdateConnectorRequest),
    DeleteConnector(MqttDeleteConnectorRequest),

    // schema
    ListSchema(MqttListSchemaRequest),
    CreateSchema(MqttCreateSchemaRequest),
    UpdateSchema(MqttUpdateSchemaRequest),
    DeleteSchema(MqttDeleteSchemaRequest),
    ListBindSchema(MqttListBindSchemaRequest),
    BindSchema(MqttBindSchemaRequest),
    UnbindSchema(MqttUnbindSchemaRequest),
}

pub struct MqttBrokerCommand {}

impl Default for MqttBrokerCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttBrokerCommand {
    pub fn new() -> Self {
        MqttBrokerCommand {}
    }

    pub async fn start(&self, params: MqttCliCommandParam) {
        let client_pool = Arc::new(ClientPool::new(100));
        match params.action {
            MqttActionType::Status => {
                self.status(&client_pool, params.clone()).await;
            }
            MqttActionType::CreateUser(ref request) => {
                self.create_user(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteUser(ref request) => {
                self.delete_user(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListUser => {
                self.list_user(&client_pool, params.clone()).await;
            }
            MqttActionType::ListConnection => {
                self.list_connections(&client_pool, params.clone()).await;
            }
            MqttActionType::EnableSlowSubscribe(ref request) => {
                self.enable_slow_subscribe(&client_pool, params.clone(), *request)
                    .await;
            }
            MqttActionType::ListTopic(ref request) => {
                self.list_topic(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListSlowSubscribe(ref request) => {
                self.list_slow_subscribe(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::EnableFlappingDetect(ref request) => {
                self.enable_flapping_detect(&client_pool, params.clone(), *request)
                    .await;
            }
            MqttActionType::Publish(ref request) => {
                self.publish(params.clone(), request.clone()).await;
            }
            MqttActionType::Subscribe(ref request) => {
                self.subscribe(params.clone(), request.clone()).await;
            }
            MqttActionType::ListConnector(ref request) => {
                self.list_connectors(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::CreateConnector(ref request) => {
                self.create_connector(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteConnector(ref request) => {
                self.delete_connector(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::UpdateConnector(ref request) => {
                self.update_connector(&client_pool, params.clone(), request.clone())
                    .await;
            }

            // schema
            MqttActionType::ListSchema(ref request) => {
                self.list_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::CreateSchema(ref request) => {
                self.create_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::UpdateSchema(ref request) => {
                self.update_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::DeleteSchema(ref request) => {
                self.delete_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::BindSchema(ref request) => {
                self.bind_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::UnbindSchema(ref request) => {
                self.unbind_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }

            MqttActionType::ListBindSchema(ref request) => {
                self.list_bind_schema(&client_pool, params.clone(), request.clone())
                    .await;
            }
        }
    }
    async fn publish(&self, params: MqttCliCommandParam, args: PublishArgsRequest) {
        // stdin stream
        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();
        // mqtt publish client
        let client_id = unique_id();
        let addr = format!("tcp://{}", params.server);
        let qos = args.qos;
        let retained = args.retained;
        let cli = connect_server5(
            client_id.as_str(),
            args.username,
            args.password,
            addr.as_str(),
            false,
            false,
        );
        let topic = args.topic;
        let props = Properties::new();
        let disconnect_opts = DisconnectOptionsBuilder::new()
            .reason_code(ReasonCode::DisconnectWithWillMessage)
            .finalize();
        println!("you can post a message on the terminal:");
        let j = tokio::spawn(async move {
            loop {
                print!("> ");
                select! {
                    _ = signal::ctrl_c() => {
                        println!(" Ctrl+C detected,  Please press ENTER to end the program. ");
                        cli.disconnect(disconnect_opts).unwrap();
                        break;
                    }

                        // Read from stdin
                    line = lines.next_line() => {
                        match line {
                            Ok(Some(input)) => {
                                    println!("You typed: {}", input);

                                    let msg = MessageBuilder::new()
                                    .properties(props.clone())
                                    .payload(input)
                                    .topic(topic.clone())
                                    .qos(qos)
                                    .retained(retained)
                                    .finalize();

                                    match cli.publish(msg) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            panic!("{:?}", e);
                                        }
                                    }
                                    if retained {
                                        println!("published retained message");
                                        cli.disconnect(disconnect_opts).unwrap(); // only one message retained
                                        break;
                                    }
                            }
                            Ok(None) => {
                                println!("End of input stream.");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error reading input: {}", e);
                                break;
                            }
                    }
                    }
                }
            }
        });

        j.await.ok();
    }

    async fn subscribe(&self, params: MqttCliCommandParam, args: SubscribeArgsRequest) {
        let client_id = unique_id();
        let addr = format!("tcp://{}", params.server);
        let cli = connect_server5(
            client_id.as_str(),
            args.username,
            args.password,
            addr.as_str(),
            false,
            false,
        );
        let sub_topics = &[args.topic];
        let qos = &[args.qos];
        // subscribe
        let rx = cli.start_consuming();
        match cli.subscribe_many(sub_topics, qos) {
            Ok(_) => {
                println!("subscribe success")
            }
            Err(e) => {
                panic!("subscribe_many: {}", e)
            }
        }

        tokio::spawn(async move {
            select! {
                _ = signal::ctrl_c() => {
             println!(" Ctrl+C detected,  Please press ENTER to end the program. ");
             let disconnect_opts = DisconnectOptionsBuilder::new()
             .reason_code(ReasonCode::DisconnectWithWillMessage)
             .finalize();
             cli.disconnect(disconnect_opts).unwrap();
            }}
        });
        while let Some(msg) = rx.iter().next() {
            match msg {
                Some(msg) => {
                    let raw = msg
                        .properties()
                        .get_string_pair_at(PropertyCode::UserProperty, 0);
                    if let Some(raw) = raw {
                        if raw.0 == "retain_push_flag" && raw.1 == "true" {
                            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                            println!("Retain message: {}", payload);
                        }
                    }
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    println!("payload: {}", payload);
                }
                None => {
                    println!("End of input stream.");
                    break;
                }
            }
        }
    }
    async fn status(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ClusterStatusRequest {};
        match mqtt_broker_cluster_status(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                println!("cluster name: {}", data.cluster_name);
                println!("node list:");
                for node in data.nodes {
                    println!("- {}", node);
                }
                println!("MQTT broker cluster up and running")
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_user(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: CreateUserRequest,
    ) {
        match mqtt_broker_create_user(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create user normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_user(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: DeleteUserRequest,
    ) {
        match mqtt_broker_delete_user(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete user normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_user(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListUserRequest {};
        match mqtt_broker_list_user(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                // format table
                let mut table = Table::new();
                table.add_row(row!["username", "is_superuser"]);
                for user in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(user.as_slice()).unwrap();
                    table.add_row(row![mqtt_user.username.as_str(), mqtt_user.is_superuser]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list user exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_connections(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListConnectionRequest {};
        match mqtt_broker_list_connection(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                let mut table = Table::new();

                println!("connection list:");
                table.add_row(row![
                    "connection_id",
                    "connection_type",
                    "protocol",
                    "source_addr",
                    "info",
                ]);

                for raw in data.list_connection_raw {
                    table.add_row(row![
                        raw.connection_id,
                        raw.connection_type,
                        raw.protocol,
                        raw.source_addr,
                        raw.info,
                    ]);
                }
                // output cmd
                table.printstd();
            }
            Err(e) => {
                println!("MQTT broker list connection exception");
                error_info(e.to_string());
            }
        }
    }

    // flapping detect
    async fn enable_flapping_detect(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: EnableFlappingDetectRequest,
    ) {
        match mqtt_broker_enable_flapping_detect(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(reply) => {
                if reply.is_enable {
                    println!("The flapping detect feature has been successfully enabled.");
                } else {
                    println!("The flapping detect feature has been successfully closed.");
                }
            }

            Err(e) => {
                println!(
                    "The flapping detect feature failed to enable, with the specific reason being:"
                );
                error_info(e.to_string());
            }
        }
    }

    // ---------------- observability ----------------
    // ------------ slow subscribe features ----------
    async fn enable_slow_subscribe(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: EnableSlowSubscribeRequest,
    ) {
        match mqtt_broker_enable_slow_subscribe(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(reply) => {
                if reply.is_enable {
                    println!("The slow subscription feature has been successfully enabled.");
                } else {
                    println!("The slow subscription feature has been successfully closed.");
                }
            }

            Err(e) => {
                println!("The slow subscription feature failed to enable, with the specific reason being:");
                error_info(e.to_string());
            }
        }
    }

    async fn list_slow_subscribe(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListSlowSubscribeRequest,
    ) {
        let slow_subscribe_request = ListSlowSubscribeRequest {
            sub_name: cli_request.sub_name,
            list: cli_request.list,
            client_id: cli_request.client_id,
            topic: cli_request.topic,
            sort: cli_request.sort,
        };
        let sort = slow_subscribe_request.sort.clone();
        match mqtt_broker_list_slow_subscribe(
            client_pool,
            &grpc_addr(params.server),
            slow_subscribe_request,
        )
        .await
        {
            Ok(data) => {
                // sort
                let sort_type = SortType::from_str(sort.as_str()).unwrap_or(SortType::DESC);
                let mut list_slow_sub_raw = data.list_slow_subscribe_raw;
                match sort_type {
                    SortType::ASC => list_slow_sub_raw.sort_by(|a, b| a.time_ms.cmp(&b.time_ms)),
                    SortType::DESC => list_slow_sub_raw.sort_by(|a, b| b.time_ms.cmp(&a.time_ms)),
                }
                // format table
                let mut table = Table::new();
                table.add_row(row![
                    "client_id",
                    "topic",
                    "sub_name",
                    "time_ms",
                    "create_time"
                ]);
                for raw in list_slow_sub_raw {
                    table.add_row(row![
                        raw.client_id,
                        raw.topic,
                        raw.sub_name,
                        raw.time_ms,
                        raw.create_time
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list slow subscribe info exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_topic(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListTopicRequest,
    ) {
        match mqtt_broker_list_topic(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(data) => {
                println!("topic list result:");
                // format table
                let mut table = Table::new();
                table.add_row(row![
                    "topic_id",
                    "topic_name",
                    "cluster_name",
                    "is_contain_retain_message",
                ]);
                let topics = data.topics;
                for topic in topics {
                    table.add_row(row![
                        topic.topic_id,
                        topic.topic_name,
                        topic.cluster_name,
                        topic.is_contain_retain_message
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list topic exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ connectors ----------------
    async fn list_connectors(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttListConnectorRequest,
    ) {
        match mqtt_broker_list_connector(client_pool, &grpc_addr(params.server), cli_request).await
        {
            Ok(data) => {
                println!("connector list result:");
                let mut table = Table::new();

                table.add_row(row![
                    "cluster name",
                    "connector name",
                    "connector type",
                    "connector config",
                    "topic id",
                    "status",
                    "broker id",
                    "create time",
                    "update time",
                ]);

                for mqtt_connector in data.connectors {
                    let connector = MQTTConnector::decode(&mqtt_connector);
                    table.add_row(row![
                        connector.cluster_name,
                        connector.connector_name,
                        connector.connector_type,
                        connector.config,
                        connector.topic_id,
                        connector.status,
                        connector.broker_id.unwrap_or(0),
                        connector.create_time,
                        connector.update_time
                    ]);
                }

                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list connector exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_connector(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttCreateConnectorRequest,
    ) {
        match mqtt_broker_create_connector(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create connector exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_connector(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttDeleteConnectorRequest,
    ) {
        match mqtt_broker_delete_connector(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker delete connector exception");
                error_info(e.to_string());
            }
        }
    }

    async fn update_connector(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttUpdateConnectorRequest,
    ) {
        match mqtt_broker_update_connector(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(_) => {
                println!("Updated successfully!")
            }
            Err(e) => {
                println!("MQTT broker update connector exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ schema ----------------
    async fn list_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttListSchemaRequest,
    ) {
        match mqtt_broker_list_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(data) => {
                println!("schema list result:");
                for mqtt_schema in data.schemas {
                    let schema = serde_json::from_slice::<SchemaData>(&mqtt_schema).unwrap();
                    println!(
                        concat!(
                            "cluster name: {}\n",
                            "schema name: {}\n",
                            "schema type: {}\n",
                            "schema desc: {}\n",
                            "schema: {}\n"
                        ),
                        schema.cluster_name,
                        schema.name,
                        schema.schema_type,
                        schema.desc,
                        schema.schema
                    );
                }
            }
            Err(e) => {
                println!("MQTT broker list schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttCreateSchemaRequest,
    ) {
        match mqtt_broker_create_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn update_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttUpdateSchemaRequest,
    ) {
        match mqtt_broker_update_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Updated successfully!")
            }
            Err(e) => {
                println!("MQTT broker update schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttDeleteSchemaRequest,
    ) {
        match mqtt_broker_delete_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker delete schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn bind_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttBindSchemaRequest,
    ) {
        match mqtt_broker_bind_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn unbind_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttUnbindSchemaRequest,
    ) {
        match mqtt_broker_unbind_schema(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker delete schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_bind_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: MqttListBindSchemaRequest,
    ) {
        match mqtt_broker_list_bind_schema(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(data) => {
                println!("bind schema list result:");
                for mqtt_schema in data.schema_binds {
                    let schema = serde_json::from_slice::<SchemaData>(&mqtt_schema).unwrap();
                    println!(
                        concat!(
                            "cluster name: {}\n",
                            "schema name: {}\n",
                            "schema type: {}\n",
                            "schema desc: {}\n",
                            "schema: {}\n"
                        ),
                        schema.cluster_name,
                        schema.name,
                        schema.schema_type,
                        schema.desc,
                        schema.schema
                    );
                }
            }
            Err(e) => {
                println!("MQTT broker list bind schema exception");
                error_info(e.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common_base::error::common::CommonError;
    use protocol::broker_mqtt::broker_mqtt_admin::{ListSlowSubScribeRaw, ListSlowSubscribeReply};

    fn set_up_slow_sub_config() -> Result<ListSlowSubscribeReply, CommonError> {
        let mut list_slow_sub_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
        let raw1 = ListSlowSubScribeRaw {
            client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
            topic: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            time_ms: 543,
            node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
            create_time: 1733898597,
            sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
        };
        list_slow_sub_raw.push(raw1);
        let raw3 = ListSlowSubScribeRaw {
            client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
            topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            time_ms: 13,
            node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
            create_time: 1733898601,
            sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
        };
        list_slow_sub_raw.push(raw3);
        let raw2 = ListSlowSubScribeRaw {
            client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
            topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            time_ms: 273,
            node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
            create_time: 1733898601,
            sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
        };
        list_slow_sub_raw.push(raw2);

        Ok(ListSlowSubscribeReply {
            list_slow_subscribe_raw: list_slow_sub_raw,
        })
    }
    #[test]
    fn test_get_sort_data_asc() {
        let mut reply = set_up_slow_sub_config().unwrap();
        reply
            .list_slow_subscribe_raw
            .sort_by(|a, b| a.time_ms.cmp(&b.time_ms));
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
                topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
                time_ms: 13,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898601,
                sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            },
            reply.list_slow_subscribe_raw[0]
        );
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
                topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
                time_ms: 273,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898601,
                sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            },
            reply.list_slow_subscribe_raw[1]
        );
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
                topic: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
                time_ms: 543,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898597,
                sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            },
            reply.list_slow_subscribe_raw[2]
        );
    }

    #[test]
    fn test_get_sort_data_desc() {
        let mut reply = set_up_slow_sub_config().unwrap();
        reply
            .list_slow_subscribe_raw
            .sort_by(|a, b| b.time_ms.cmp(&a.time_ms));
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
                topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
                time_ms: 13,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898601,
                sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            },
            reply.list_slow_subscribe_raw[2]
        );
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "49e10a8d8a494cefa904a00dcf0b30af".to_string(),
                topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
                time_ms: 273,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898601,
                sub_name: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            },
            reply.list_slow_subscribe_raw[1]
        );
        assert_eq!(
            ListSlowSubScribeRaw {
                client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
                topic: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
                time_ms: 543,
                node_info: "RobustMQ-MQTT@172.22.194.185".to_string(),
                create_time: 1733898597,
                sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            },
            reply.list_slow_subscribe_raw[0]
        );
    }
}
