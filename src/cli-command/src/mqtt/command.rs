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

use crate::mqtt::pub_sub::{connect_server5, error_info};
use crate::mqtt::pub_sub::{PublishArgsRequest, SubscribeArgsRequest};
use admin_server::client::AdminHttpClient;
use admin_server::response::mqtt::SessionListRow;
use common_base::tools::unique_id;
use paho_mqtt::{DisconnectOptionsBuilder, MessageBuilder, Properties, PropertyCode, ReasonCode};
use prettytable::{row, Table};

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
    // session
    ListSession,

    // subscribe
    ListSubscribe,

    // user admin
    ListUser,
    CreateUser(admin_server::request::mqtt::CreateUserReq),
    DeleteUser(admin_server::request::mqtt::DeleteUserReq),

    // access control list admin
    ListAcl,
    CreateAcl(admin_server::request::mqtt::CreateAclReq),
    DeleteAcl(admin_server::request::mqtt::DeleteAclReq),

    // blacklist admin
    ListBlacklist,
    CreateBlacklist(admin_server::request::mqtt::CreateBlackListReq),
    DeleteBlacklist(admin_server::request::mqtt::DeleteBlackListReq),

    // client
    ListClient,

    // #### observability ####
    // slow subscribe
    ListSlowSubscribe,

    // system alarm
    ListSystemAlarm,

    // topic rewrite rule
    ListTopicRewrite,
    CreateTopicRewrite(admin_server::request::mqtt::CreateTopicRewriteReq),
    DeleteTopicRewrite(admin_server::request::mqtt::DeleteTopicRewriteReq),

    // publish
    Publish(PublishArgsRequest),

    // subscribe
    Subscribe(SubscribeArgsRequest),

    // Topic
    ListTopic,

    // flapping detect
    ListFlappingDetect,

    // connector
    ListConnector,
    CreateConnector(admin_server::request::mqtt::CreateConnectorReq),
    DeleteConnector(admin_server::request::mqtt::DeleteConnectorReq),

    // schema
    ListSchema,
    CreateSchema(admin_server::request::mqtt::CreateSchemaReq),
    DeleteSchema(admin_server::request::mqtt::DeleteSchemaReq),
    ListBindSchema,
    BindSchema(admin_server::request::mqtt::CreateSchemaBindReq),
    UnbindSchema(admin_server::request::mqtt::DeleteSchemaBindReq),

    //auto subscribe
    ListAutoSubscribe,
    CreateAutoSubscribe(admin_server::request::mqtt::CreateAutoSubscribeReq),
    DeleteAutoSubscribe(admin_server::request::mqtt::DeleteAutoSubscribeReq),
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
        let params_clone = params.clone();
        match params.action {
            // client
            MqttActionType::ListClient => {
                self.list_clients(params.clone()).await;
            }

            // session
            MqttActionType::ListSession => {
                self.list_session(params.clone()).await;
            }

            // topic
            MqttActionType::ListTopic => {
                self.list_topic(params.clone()).await;
            }

            // topic rewrite
            MqttActionType::ListTopicRewrite => {
                self.list_topic_rewrite_rule(params.clone()).await;
            }

            MqttActionType::CreateTopicRewrite(request) => {
                self.create_topic_rewrite_rule(params_clone.clone(), request)
                    .await;
            }

            MqttActionType::DeleteTopicRewrite(request) => {
                self.delete_topic_rewrite_rule(params_clone.clone(), request)
                    .await;
            }

            // subscribe
            MqttActionType::ListSubscribe => {
                self.list_subscribe(params_clone.clone()).await;
            }

            //auto subscribe
            MqttActionType::ListAutoSubscribe => {
                self.list_auto_subscribe_rule(params_clone.clone()).await;
            }
            MqttActionType::CreateAutoSubscribe(request) => {
                self.set_auto_subscribe_rule(params_clone.clone(), request)
                    .await;
            }
            MqttActionType::DeleteAutoSubscribe(request) => {
                self.delete_auto_subscribe_rule(params_clone.clone(), request)
                    .await;
            }

            // slow subscribe
            MqttActionType::ListSlowSubscribe => {
                self.list_slow_subscribe(params_clone.clone()).await;
            }

            // system alarm
            MqttActionType::ListSystemAlarm => {
                self.list_system_alarm(params_clone.clone()).await;
            }

            // user
            MqttActionType::ListUser => {
                self.list_user(params_clone.clone()).await;
            }
            MqttActionType::CreateUser(request) => {
                self.create_user(params_clone.clone(), request).await;
            }
            MqttActionType::DeleteUser(request) => {
                self.delete_user(params_clone.clone(), request).await;
            }

            // acl
            MqttActionType::ListAcl => {
                self.list_acl(params_clone.clone()).await;
            }
            MqttActionType::CreateAcl(request) => {
                self.create_acl(params_clone.clone(), request).await;
            }
            MqttActionType::DeleteAcl(request) => {
                self.delete_acl(params_clone.clone(), request).await;
            }

            // blacklist
            MqttActionType::ListBlacklist => {
                self.list_blacklist(params_clone.clone()).await;
            }
            MqttActionType::CreateBlacklist(request) => {
                self.create_blacklist(params_clone.clone(), request).await;
            }
            MqttActionType::DeleteBlacklist(request) => {
                self.delete_blacklist(params_clone.clone(), request).await;
            }

            // flapping detect
            MqttActionType::ListFlappingDetect => {
                self.list_flapping_detect(params_clone.clone()).await;
            }

            // connector
            MqttActionType::ListConnector => {
                self.list_connectors(params_clone.clone()).await;
            }
            MqttActionType::CreateConnector(request) => {
                self.create_connector(params_clone.clone(), request).await;
            }
            MqttActionType::DeleteConnector(request) => {
                self.delete_connector(params_clone.clone(), request).await;
            }

            // schema
            MqttActionType::ListSchema => {
                self.list_schema(params_clone.clone()).await;
            }
            MqttActionType::CreateSchema(request) => {
                self.create_schema(params_clone.clone(), request).await;
            }
            MqttActionType::DeleteSchema(request) => {
                self.delete_schema(params_clone.clone(), request).await;
            }

            MqttActionType::ListBindSchema => {
                self.list_bind_schema(params_clone.clone()).await;
            }
            MqttActionType::BindSchema(request) => {
                self.bind_schema(params_clone.clone(), request).await;
            }
            MqttActionType::UnbindSchema(request) => {
                self.unbind_schema(params_clone.clone(), request).await;
            }

            // pub && sub
            MqttActionType::Publish(ref request) => {
                self.publish(params.clone(), request.clone()).await;
            }
            MqttActionType::Subscribe(ref request) => {
                self.subscribe(params.clone(), request.clone()).await;
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

    // ------------ list session ------------
    async fn list_session(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for session list
        let request = admin_server::request::mqtt::SessionListReq {
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
            .get_session_list::<admin_server::request::mqtt::SessionListReq, Vec<SessionListRow>>(
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateUserReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_user(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteUserReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_user(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete user normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_user(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for user list
        let request = admin_server::request::mqtt::UserListReq {
            user_name: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_user_list::<admin_server::request::mqtt::UserListReq, Vec<admin_server::response::mqtt::UserListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("user list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row!["username", "is_superuser"]);
                for user in page_data.data {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateAclReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_acl(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteAclReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_acl(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete acl normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_acl(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for acl list
        let request = admin_server::request::mqtt::AclListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_acl_list::<admin_server::request::mqtt::AclListReq, Vec<admin_server::response::mqtt::AclListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("acl list result:");
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
                for acl in page_data.data {
                    table.add_row(row![
                        acl.resource_type,
                        acl.resource_name,
                        acl.topic,
                        acl.ip,
                        acl.action,
                        acl.permission
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateBlackListReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_blacklist(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteBlackListReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_blacklist(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete blacklist normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_blacklist(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for blacklist list
        let request = admin_server::request::mqtt::BlackListListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_blacklist::<admin_server::request::mqtt::BlackListListReq, Vec<admin_server::response::mqtt::BlackListListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("blacklist list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "blacklist_type",
                    "resource_name",
                    "end_time",
                    "description"
                ]);
                for blacklist in page_data.data {
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
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for client list
        let request = admin_server::request::mqtt::ClientListReq {
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
            .get_client_list::<admin_server::request::mqtt::ClientListReq, Vec<admin_server::response::mqtt::ClientListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                let mut table = Table::new();

                println!("connection list:");
                table.set_titles(row![
                    "client_id",
                    "connection_id",
                    "protocol",
                    "create_time",
                ]);

                for client in page_data.data {
                    let network_conn =  client.network_connection.unwrap();
                    table.add_row(row![
                        client.connection_id,
                        client.client_id,
                        network_conn.protocol.unwrap().to_str(),
                        client.mqtt_connection.create_time,
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
    async fn list_flapping_detect(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for flapping detect list
        let request = admin_server::request::mqtt::SystemAlarmListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_flapping_detect_list::<admin_server::request::mqtt::SystemAlarmListReq, Vec<admin_server::response::mqtt::FlappingDetectListRaw>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("flapping detect list result:");
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "before_last_windows_connections",
                    "first_request_time",
                ]);
                for raw in page_data.data {
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
                println!("MQTT broker list flapping detect exception");
                error_info(e.to_string());
            }
        }
    }

    // #### observability ###
    // ---- slow subscribe ----

    async fn list_slow_subscribe(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for slow subscribe list
        let request = admin_server::request::mqtt::AutoSubscribeListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_slow_subscribe_list::<admin_server::request::mqtt::AutoSubscribeListReq, Vec<admin_server::response::mqtt::SlowSubscribeListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("slow subscribe list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "client_id",
                    "topic_name",
                    "subscribe_name",
                    "time_span",
                    "create_time"
                ]);
                for raw in page_data.data {
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

    async fn list_topic(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for topic list
        let request = admin_server::request::mqtt::TopicListReq {
            topic_name: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_topic_list::<admin_server::request::mqtt::TopicListReq, Vec<admin_server::response::mqtt::TopicListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("topic list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "topic_id",
                    "topic_name",
                    "is_contain_retain_message",
                ]);
                for topic in page_data.data {
                    table.add_row(row![
                        topic.topic_id,
                        topic.topic_name,
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
    async fn list_system_alarm(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for system alarm list
        let request = admin_server::request::mqtt::SystemAlarmListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_system_alarm_list::<admin_server::request::mqtt::SystemAlarmListReq, Vec<admin_server::response::mqtt::SystemAlarmListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("system alarm list result:");
                let mut table = Table::new();
                table.set_titles(row!["name", "message", "create_time"]);
                for alarm in page_data.data {
                    table.add_row(row![
                        alarm.name,
                        alarm.message,
                        alarm.create_time,
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
    async fn list_subscribe(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for subscribe list
        let request = admin_server::request::mqtt::SubscribeListReq {
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
            .get_subscribe_list::<admin_server::request::mqtt::SubscribeListReq, Vec<admin_server::response::mqtt::SubscribeListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("subscribe list result:");
                // format table
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
                for raw in page_data.data {
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

    // ------------------ connectors ----------------
    async fn list_connectors(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for connector list
        let request = admin_server::request::mqtt::ConnectorListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_connector_list::<admin_server::request::mqtt::ConnectorListReq, Vec<admin_server::response::mqtt::ConnectorListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("connector list result:");
                let mut table = Table::new();

                table.set_titles(row![
                    "connector name",
                    "connector type",
                    "connector config",
                    "topic id",
                    "status",
                    "broker id",
                    "create time",
                    "update time",
                ]);

                for connector in page_data.data {
                    table.add_row(row![
                        connector.connector_name,
                        connector.connector_type,
                        connector.config,
                        connector.topic_id,
                        connector.status,
                        connector.broker_id,
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateConnectorReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_connector(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteConnectorReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_connector(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker delete connector exception");
                error_info(e.to_string());
            }
        }
    }

    // ------------------ topic rewrite rule ----------------
    async fn list_topic_rewrite_rule(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for topic rewrite rule list
        let request = admin_server::request::mqtt::TopicRewriteReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_topic_rewrite_list::<admin_server::request::mqtt::TopicRewriteReq, Vec<admin_server::response::mqtt::TopicRewriteListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("topic rewrite rule list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "source_topic",
                    "dest_topic",
                    "action",
                    "regex",
                ]);
                for rule in page_data.data {
                    table.add_row(row![
                        rule.source_topic,
                        rule.dest_topic,
                        rule.action,
                        rule.regex
                    ]);
                }
                // output cmd
                table.printstd()
            }
            Err(e) => {
                println!("MQTT broker list topic rewrite rule exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_topic_rewrite_rule(
        &self,
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateTopicRewriteReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_topic_rewrite(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteTopicRewriteReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_topic_rewrite(&cli_request).await {
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
    async fn list_schema(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for schema list
        let request = admin_server::request::mqtt::SchemaListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_schema_list::<admin_server::request::mqtt::SchemaListReq, Vec<admin_server::response::mqtt::SchemaListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("schema list result:");
                for schema in page_data.data {
                    println!(
                        concat!(
                            "schema name: {}\n",
                            "schema type: {}\n",
                            "schema desc: {}\n",
                            "schema: {}\n"
                        ),
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateSchemaReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_schema(&cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker create schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_schema(
        &self,
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteSchemaReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_schema(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateSchemaBindReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_schema_bind(&cli_request).await {
            Ok(_) => {
                println!("Created successfully!")
            }
            Err(e) => {
                println!("MQTT broker bind schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn unbind_schema(
        &self,
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteSchemaBindReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_schema_bind(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!")
            }
            Err(e) => {
                println!("MQTT broker unbind schema exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_bind_schema(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for schema bind list
        let request = admin_server::request::mqtt::SchemaBindListReq {
            resource_name: None,
            schema_name: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_schema_bind_list::<admin_server::request::mqtt::SchemaBindListReq, Vec<admin_server::response::mqtt::SchemaBindListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("bind schema list result:");
                for bind in page_data.data {
                    println!("data type: {}", bind.data_type);
                    for data_item in bind.data {
                        println!("  - {data_item}");
                    }
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::CreateAutoSubscribeReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.create_auto_subscribe(&cli_request).await {
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
        params: MqttCliCommandParam,
        cli_request: admin_server::request::mqtt::DeleteAutoSubscribeReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.delete_auto_subscribe(&cli_request).await {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete auto subscribe rule normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_auto_subscribe_rule(&self, params: MqttCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        // Create request for auto subscribe rule list
        let request = admin_server::request::mqtt::AutoSubscribeListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_auto_subscribe_list::<admin_server::request::mqtt::AutoSubscribeListReq, Vec<admin_server::response::mqtt::AutoSubscribeListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("auto subscribe rule list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "topic",
                    "qos",
                    "no_local",
                    "retain_as_published",
                    "retained_handling",
                ]);
                for rule in page_data.data {
                    table.add_row(row![
                        rule.topic,
                        rule.qos,
                        rule.no_local,
                        rule.retain_as_published,
                        rule.retained_handling
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
