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
use admin_server::response::SessionListRow;
use common_base::enum_type::sort_type::SortType;
use common_base::tools::unique_id;
use common_config::config::BrokerConfig;
use grpc_clients::mqtt::admin::call::{
    mqtt_broker_bind_schema, mqtt_broker_create_acl, mqtt_broker_create_blacklist,
    mqtt_broker_create_connector, mqtt_broker_create_schema, mqtt_broker_create_topic_rewrite_rule,
    mqtt_broker_create_user, mqtt_broker_delete_acl, mqtt_broker_delete_auto_subscribe_rule,
    mqtt_broker_delete_blacklist, mqtt_broker_delete_connector, mqtt_broker_delete_schema,
    mqtt_broker_delete_topic_rewrite_rule, mqtt_broker_delete_user,
    mqtt_broker_enable_flapping_detect, mqtt_broker_list_acl, mqtt_broker_list_auto_subscribe_rule,
    mqtt_broker_list_bind_schema, mqtt_broker_list_blacklist, mqtt_broker_list_connector,
    mqtt_broker_list_flapping_detect, mqtt_broker_list_schema, mqtt_broker_list_slow_subscribe,
    mqtt_broker_list_subscribe, mqtt_broker_list_system_alarm, mqtt_broker_list_topic,
    mqtt_broker_list_user, mqtt_broker_set_auto_subscribe_rule, mqtt_broker_set_cluster_config,
    mqtt_broker_set_slow_subscribe_config, mqtt_broker_set_system_alarm_config,
    mqtt_broker_subscribe_detail, mqtt_broker_unbind_schema, mqtt_broker_update_connector,
    mqtt_broker_update_schema,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::schema::SchemaData;
use paho_mqtt::{DisconnectOptionsBuilder, MessageBuilder, Properties, PropertyCode, ReasonCode};
use prettytable::{row, Table};
use protocol::broker::broker_mqtt_admin::{
    BindSchemaRequest, CreateAclRequest, CreateBlacklistRequest, CreateConnectorRequest,
    CreateSchemaRequest, CreateTopicRewriteRuleRequest, CreateUserRequest, DeleteAclRequest,
    DeleteAutoSubscribeRuleRequest, DeleteBlacklistRequest, DeleteConnectorRequest,
    DeleteSchemaRequest, DeleteTopicRewriteRuleRequest, DeleteUserRequest,
    EnableFlappingDetectRequest, ListAclRequest, ListAutoSubscribeRuleRequest,
    ListBindSchemaRequest, ListBlacklistRequest, ListConnectorRequest, ListFlappingDetectRequest,
    ListSchemaRequest, ListSlowSubscribeRequest, ListSubscribeRequest, ListSystemAlarmRequest,
    ListTopicRequest, ListUserRequest, SetAutoSubscribeRuleRequest, SetClusterConfigRequest,
    SetSlowSubscribeConfigRequest, SetSystemAlarmConfigRequest, SubscribeDetailRequest,
    UnbindSchemaRequest, UpdateConnectorRequest, UpdateSchemaRequest,
};
use std::str::FromStr;
use std::sync::Arc;

use metadata_struct::acl::mqtt_acl::MqttAcl;

// Default pagination constants
const DEFAULT_PAGE_SIZE: u32 = 10000;
const DEFAULT_PAGE_NUM: u32 = 1;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::{select, signal};

#[derive(Clone)]
pub struct MqttCliCommandParam {
    pub server: String,
    pub action: MqttActionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MqttActionType {
    // common
    SetClusterConfig(SetClusterConfigRequest),

    // cluster config
    GetClusterConfig,

    // session
    ListSession,

    // subscribe
    ListSubscribe(ListSubscribeRequest),
    DetailSubscribe(SubscribeDetailRequest),

    // user admin
    ListUser,
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),

    // access control list admin
    ListAcl,
    CreateAcl(CreateAclRequest),
    DeleteAcl(DeleteAclRequest),

    // blacklist admin
    ListBlacklist,
    CreateBlacklist(CreateBlacklistRequest),
    DeleteBlacklist(DeleteBlacklistRequest),

    // client
    ListClient,

    // #### observability ####
    // slow subscribe
    SetSlowSubscribeConfig(SetSlowSubscribeConfigRequest),
    ListSlowSubscribe(ListSlowSubscribeRequest),

    // flapping detect
    EnableFlappingDetect(EnableFlappingDetectRequest),

    // system alarm
    SetSystemAlarmConfig(SetSystemAlarmConfigRequest),
    ListSystemAlarm(ListSystemAlarmRequest),

    // topic rewrite rule
    CreateTopicRewriteRule(CreateTopicRewriteRuleRequest),
    DeleteTopicRewriteRule(DeleteTopicRewriteRuleRequest),

    // publish
    Publish(PublishArgsRequest),

    // subscribe
    Subscribe(SubscribeArgsRequest),

    ListTopic,
    ListFlappingDetect,

    // connector
    ListConnector(ListConnectorRequest),
    CreateConnector(CreateConnectorRequest),
    UpdateConnector(UpdateConnectorRequest),
    DeleteConnector(DeleteConnectorRequest),

    // schema
    ListSchema(ListSchemaRequest),
    CreateSchema(CreateSchemaRequest),
    UpdateSchema(UpdateSchemaRequest),
    DeleteSchema(DeleteSchemaRequest),
    ListBindSchema(ListBindSchemaRequest),
    BindSchema(BindSchemaRequest),
    UnbindSchema(UnbindSchemaRequest),

    //auto subscribe
    ListAutoSubscribeRule(ListAutoSubscribeRuleRequest),
    SetAutoSubscribeRule(SetAutoSubscribeRuleRequest),
    DeleteAutoSubscribeRule(DeleteAutoSubscribeRuleRequest),
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
            // cluster config
            MqttActionType::GetClusterConfig => {
                self.get_cluster_config(params.clone()).await;
            }

            // list connection
            MqttActionType::ListClient => {
                self.list_clients(params.clone()).await;
            }

            // list session
            MqttActionType::ListSession => {
                self.list_session(params.clone()).await;
            }

            // user admin
            MqttActionType::ListUser => {
                self.list_user(&client_pool, params.clone()).await;
            }
            MqttActionType::CreateUser(ref request) => {
                self.create_user(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteUser(ref request) => {
                self.delete_user(&client_pool, params.clone(), request.clone())
                    .await;
            }
            // access control list admin
            MqttActionType::ListAcl => {
                self.list_acl(&client_pool, params.clone()).await;
            }
            MqttActionType::CreateAcl(ref request) => {
                self.create_acl(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteAcl(ref request) => {
                self.delete_acl(&client_pool, params.clone(), request.clone())
                    .await;
            }

            // blacklist admin
            MqttActionType::ListBlacklist => {
                self.list_blacklist(&client_pool, params.clone()).await;
            }
            MqttActionType::CreateBlacklist(ref request) => {
                self.create_blacklist(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteBlacklist(ref request) => {
                self.delete_blacklist(&client_pool, params.clone(), request.clone())
                    .await;
            }

            // connector
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
            // topic rewrite rule
            MqttActionType::CreateTopicRewriteRule(ref request) => {
                self.create_topic_rewrite_rule(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteTopicRewriteRule(ref request) => {
                self.delete_topic_rewrite_rule(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListTopic => {
                self.list_topic(&client_pool, params.clone()).await;
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

            //auto subscribe
            MqttActionType::ListAutoSubscribeRule(ref request) => {
                self.list_auto_subscribe_rule(&client_pool, params.clone(), *request)
                    .await;
            }
            MqttActionType::SetAutoSubscribeRule(ref request) => {
                self.set_auto_subscribe_rule(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteAutoSubscribeRule(ref request) => {
                self.delete_auto_subscribe_rule(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::SetClusterConfig(ref request) => {
                self.set_cluster_config(&client_pool, params.clone(), request.clone())
                    .await;
            }
            MqttActionType::SetSystemAlarmConfig(ref request) => {
                self.set_system_alarm_config(&client_pool, params.clone(), *request)
                    .await;
            }
            MqttActionType::ListSystemAlarm(ref request) => {
                self.list_system_alarm(&client_pool, params.clone(), *request)
                    .await;
            }
            MqttActionType::ListFlappingDetect => {
                self.list_flapping_detect(&client_pool, params.clone())
                    .await
            }

            // subscribe
            MqttActionType::ListSubscribe(ref request) => {
                self.list_subscribe(&client_pool, params.clone(), request.to_owned())
                    .await;
            }
            MqttActionType::DetailSubscribe(ref request) => {
                self.detail_subscribe(&client_pool, params.clone(), request.to_owned())
                    .await;
            }
            MqttActionType::SetSlowSubscribeConfig(ref request) => {
                self.set_slow_subscribe_config(&client_pool, params.clone(), request.to_owned())
                    .await
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
                                    println!("You typed: {input}");

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
                                            panic!("{e:?}");
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
                                eprintln!("Error reading input: {e}");
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
                panic!("subscribe_many: {e}")
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
                            println!("Retain message: {payload}");
                        }
                    }
                    let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
                    println!("payload: {payload}");
                }
                None => {
                    println!("End of input stream.");
                    break;
                }
            }
        }
    }

    // ------------ common -------------
    async fn set_cluster_config(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: SetClusterConfigRequest,
    ) {
        match mqtt_broker_set_cluster_config(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(reply) => {
                let feature_name = reply.feature_name.as_str();
                if reply.is_enable {
                    println!("Enabled successfully! feature name: {feature_name}");
                } else {
                    println!("Disabled successfully! feature name: {feature_name}");
                }
            }
            Err(e) => {
                println!("MQTT broker enable feature normal exception: {e}");
                error_info(e.to_string());
            }
        }
    }

    async fn get_cluster_config(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client =
            crate::admin_client::AdminHttpClient::new(format!("http://{}", params.server));

        // Create empty request for get cluster config
        let request = serde_json::json!({});

        match admin_client.get_cluster_config(&request).await {
            Ok(response_text) => {
                // Try to parse the response as BrokerConfig
                match serde_json::from_str::<BrokerConfig>(&response_text) {
                    Ok(data) => {
                        let json = match serde_json::to_string_pretty(&data) {
                            Ok(data) => data,
                            Err(e) => {
                                println!("MQTT broker cluster normal exception");
                                error_info(e.to_string());
                                return;
                            }
                        };
                        println!("{json}");
                    }
                    Err(_) => {
                        // If direct parsing fails, try to parse as the original response format
                        println!("{response_text}");
                    }
                }
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------ list session ------------
    async fn list_session(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client =
            crate::admin_client::AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for session list
        let request = admin_server::request::SessionListReq {
            client_id: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_session_list::<admin_server::request::SessionListReq, Vec<SessionListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "session_expiry",
                    "is_contain_last_will",
                    "last_will_delay_interval",
                    "create_time",
                    "connection_id",
                    "broker_id",
                    "reconnect_time",
                    "distinct_time"
                ]);
                for session in page_data.data {
                    table.add_row(row![
                        session.client_id,
                        session.session_expiry,
                        session.is_contain_last_will,
                        session.last_will_delay_interval.unwrap_or_default(),
                        session.create_time,
                        session.connection_id.unwrap_or_default(),
                        session.broker_id.unwrap_or_default(),
                        session.reconnect_time.unwrap_or_default(),
                        session.distinct_time.unwrap_or_default(),
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------ user admin ------------
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
                table.set_titles(row!["username", "is_superuser"]);
                for user in data.users {
                    table.add_row(row![user.username.as_str(), user.is_superuser]);
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
    // -------------- acl admin --------------

    async fn create_acl(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: CreateAclRequest,
    ) {
        match mqtt_broker_create_acl(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create acl normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_acl(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: DeleteAclRequest,
    ) {
        match mqtt_broker_delete_acl(client_pool, &grpc_addr(params.server), cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete acl normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_acl(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListAclRequest::default();
        match mqtt_broker_list_acl(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "resource_type",
                    "resource_name",
                    "topic",
                    "ip",
                    "action",
                    "permission"
                ]);
                for acl in data.acls {
                    let mqtt_acl = serde_json::from_slice::<MqttAcl>(acl.as_slice()).unwrap();
                    table.add_row(row![
                        mqtt_acl.resource_type,
                        mqtt_acl.resource_name,
                        mqtt_acl.topic,
                        mqtt_acl.ip,
                        mqtt_acl.action,
                        mqtt_acl.permission
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list acl exception");
                error_info(e.to_string());
            }
        }
    }

    // -------------- blacklist admin --------------
    async fn create_blacklist(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: CreateBlacklistRequest,
    ) {
        match mqtt_broker_create_blacklist(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create blacklist normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_blacklist(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: DeleteBlacklistRequest,
    ) {
        match mqtt_broker_delete_blacklist(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete blacklist normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_blacklist(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListBlacklistRequest::default();
        match mqtt_broker_list_blacklist(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "blacklist_type",
                    "resource_name",
                    "end_time",
                    "description"
                ]);
                for blacklist in data.blacklists {
                    table.add_row(row![
                        blacklist.blacklist_type,
                        blacklist.resource_name,
                        blacklist.end_time,
                        blacklist.desc
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list blacklist exception");
                error_info(e.to_string());
            }
        }
    }

    // -------------- list connections --------------
    async fn list_clients(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client =
            crate::admin_client::AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for client list
        let request = admin_server::request::ClientListReq {
            source_ip: None,
            connection_id: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_client_list::<admin_server::request::ClientListReq, Vec<admin_server::response::ClientListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                let mut table = Table::new();

                println!("connection list:");
                table.set_titles(row![
                    "connection_id",
                    "connection_type",
                    "protocol",
                    "source_addr",
                    "create_time",
                ]);

                for client in page_data.data {
                    table.add_row(row![
                        client.connection_id,
                        client.connection_type,
                        client.protocol,
                        client.source_addr,
                        client.create_time,
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

    // -------------- flapping detect --------------
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

    async fn list_flapping_detect(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListFlappingDetectRequest {};
        match mqtt_broker_list_flapping_detect(client_pool, &grpc_addr(params.server), request)
            .await
        {
            Ok(data) => {
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "before_last_windows_connections",
                    "first_request_time",
                ]);
                for raw in data.flapping_detect_raw {
                    table.add_row(row![
                        raw.client_id,
                        raw.before_last_windows_connections,
                        raw.first_request_time
                    ]);
                }

                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    // #### observability ###
    // ---- slow subscribe ----

    async fn set_slow_subscribe_config(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: SetSlowSubscribeConfigRequest,
    ) {
        match mqtt_broker_set_slow_subscribe_config(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(data) => {
                println!("Set slow subscribe config successfully! Current Config:");
                let mut table = Table::new();
                table.set_titles(row!["Config Options", "Value"]);
                table.add_row(row!["enable", data.is_enable]);
                table.add_row(row!["delay_type", data.delay_type]);
                table.add_row(row!["max_store_num", data.max_store_num]);
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker set slow subscribe config exception");
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
                    SortType::ASC => {
                        list_slow_sub_raw.sort_by(|a, b| a.time_span.cmp(&b.time_span))
                    }
                    SortType::DESC => {
                        list_slow_sub_raw.sort_by(|a, b| b.time_span.cmp(&a.time_span))
                    }
                }
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "topic_name",
                    "subscribe_name",
                    "time_span",
                    "create_time"
                ]);
                for raw in list_slow_sub_raw {
                    table.add_row(row![
                        raw.client_id,
                        raw.topic_name,
                        raw.subscribe_name,
                        raw.time_span,
                        raw.create_time,
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

    async fn list_topic(&self, client_pool: &ClientPool, params: MqttCliCommandParam) {
        let request = ListTopicRequest { topic_name: None };
        match mqtt_broker_list_topic(client_pool, &grpc_addr(params.server), request).await {
            Ok(data) => {
                println!("topic list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
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

    // ---- system alarms ----
    async fn set_system_alarm_config(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: SetSystemAlarmConfigRequest,
    ) {
        match mqtt_broker_set_system_alarm_config(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(data) => {
                println!("Set system alarm config successfully! Current Config:");
                let mut table = Table::new();
                table.set_titles(row!["Config Options", "Value"]);
                table.add_row(row!["enable", data.enable]);
                if let Some(memory_high_watermark) = data.os_memory_high_watermark {
                    table.add_row(row!["memory-high-watermark", memory_high_watermark]);
                }
                if let Some(cpu_high_watermark) = data.os_cpu_high_watermark {
                    table.add_row(row!["cpu-high-watermark", cpu_high_watermark]);
                }
                if let Some(cpu_low_watermark) = data.os_cpu_low_watermark {
                    table.add_row(row!["cpu-low-watermark", cpu_low_watermark]);
                }
                if let Some(cpu_check_interval_ms) = data.os_cpu_check_interval_ms {
                    table.add_row(row!["cpu-check-interval-ms", cpu_check_interval_ms]);
                }

                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker set system alarm config exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_system_alarm(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListSystemAlarmRequest,
    ) {
        match mqtt_broker_list_system_alarm(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(data) => {
                println!("system alarm list result:");
                let mut table = Table::new();
                table.set_titles(row!["name", "message", "activate_at", "activated"]);
                for alarm in data.list_system_alarm_raw {
                    table.add_row(row![
                        alarm.name,
                        alarm.message,
                        alarm.activate_at,
                        alarm.activated
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list system alarm exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ subscribe ----------------
    async fn list_subscribe(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListSubscribeRequest,
    ) {
        match mqtt_broker_list_subscribe(client_pool, &grpc_addr(params.server), cli_request).await
        {
            Ok(data) => {
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "is_share_sub",
                    "path",
                    "broker_id",
                    "protocol",
                    "qos",
                    "no_local",
                    "preserve_retain",
                    "retain_handling",
                    "create_time",
                    "pk_id",
                    "properties"
                ]);
                for raw in data.subscriptions {
                    table.add_row(row![
                        raw.client_id,
                        raw.is_share_sub,
                        raw.path,
                        raw.broker_id,
                        raw.protocol,
                        raw.qos,
                        raw.no_local,
                        raw.preserve_retain,
                        raw.retain_handling,
                        raw.create_time,
                        raw.pk_id,
                        raw.properties
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list subscribe exception");
                error_info(e.to_string());
            }
        }
    }

    async fn detail_subscribe(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: SubscribeDetailRequest,
    ) {
        match mqtt_broker_subscribe_detail(client_pool, &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(data) => {
                println!("subscribe info:{}", data.sub_info);
                for raw in data.details {
                    println!("=======================");
                    println!("sub:{}", raw.sub);
                    println!("thread:{}", raw.thread);
                    println!("=======================");
                }
            }
            Err(e) => {
                println!("MQTT broker detail subscribe exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ connectors ----------------
    async fn list_connectors(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListConnectorRequest,
    ) {
        match mqtt_broker_list_connector(client_pool, &grpc_addr(params.server), cli_request).await
        {
            Ok(data) => {
                println!("connector list result:");
                let mut table = Table::new();

                table.set_titles(row![
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

                for connector in data.connectors {
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
        cli_request: CreateConnectorRequest,
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
        cli_request: DeleteConnectorRequest,
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
        cli_request: UpdateConnectorRequest,
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

    // ------------------ topic rewrite rule ----------------
    async fn create_topic_rewrite_rule(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: CreateTopicRewriteRuleRequest,
    ) {
        match mqtt_broker_create_topic_rewrite_rule(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create topic rewrite rule exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_topic_rewrite_rule(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: DeleteTopicRewriteRuleRequest,
    ) {
        match mqtt_broker_delete_topic_rewrite_rule(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker delete topic rewrite rule exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ schema ----------------
    async fn list_schema(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListSchemaRequest,
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
        cli_request: CreateSchemaRequest,
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
        cli_request: UpdateSchemaRequest,
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
        cli_request: DeleteSchemaRequest,
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
        cli_request: BindSchemaRequest,
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
        cli_request: UnbindSchemaRequest,
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
        cli_request: ListBindSchemaRequest,
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

    // ------------------ auto subscribe ----------------
    async fn set_auto_subscribe_rule(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: SetAutoSubscribeRuleRequest,
    ) {
        match mqtt_broker_set_auto_subscribe_rule(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker set auto subscribe rule normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_auto_subscribe_rule(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: DeleteAutoSubscribeRuleRequest,
    ) {
        match mqtt_broker_delete_auto_subscribe_rule(
            client_pool,
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete auto subscribe rule normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_auto_subscribe_rule(
        &self,
        client_pool: &ClientPool,
        params: MqttCliCommandParam,
        cli_request: ListAutoSubscribeRuleRequest,
    ) {
        let _ = cli_request;
        let request = ListAutoSubscribeRuleRequest {};
        match mqtt_broker_list_auto_subscribe_rule(client_pool, &grpc_addr(params.server), request)
            .await
        {
            Ok(data) => {
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "topic",
                    "qos",
                    "no_local",
                    "retain_as_published",
                    "retained_handling",
                ]);
                for rule in data.auto_subscribe_rules {
                    let mqtt_auto_subscribe_rule =
                        match serde_json::from_slice::<MqttAutoSubscribeRule>(rule.as_slice()) {
                            Ok(rule) => rule,
                            Err(e) => {
                                error_info(e.to_string());
                                continue;
                            }
                        };
                    table.add_row(row![
                        mqtt_auto_subscribe_rule.topic,
                        Into::<u8>::into(mqtt_auto_subscribe_rule.qos),
                        mqtt_auto_subscribe_rule.no_local,
                        mqtt_auto_subscribe_rule.retain_as_published,
                        Into::<u8>::into(mqtt_auto_subscribe_rule.retained_handling)
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list auto subscribe rule exception");
                error_info(e.to_string());
            }
        }
    }
}
