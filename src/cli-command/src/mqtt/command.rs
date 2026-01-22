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
use admin_server::mqtt::session::SessionListRow;
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
    // overview
    Overview,
    // session
    ListSession,

    // subscribe
    ListSubscribe,

    // user admin
    ListUser,
    CreateUser(admin_server::mqtt::user::CreateUserReq),
    DeleteUser(admin_server::mqtt::user::DeleteUserReq),

    // access control list admin
    ListAcl,
    CreateAcl(admin_server::mqtt::acl::CreateAclReq),
    DeleteAcl(admin_server::mqtt::acl::DeleteAclReq),

    // blacklist admin
    ListBlacklist,
    CreateBlacklist(admin_server::mqtt::blacklist::CreateBlackListReq),
    DeleteBlacklist(admin_server::mqtt::blacklist::DeleteBlackListReq),

    // client
    ListClient,

    // #### observability ####
    // slow subscribe
    ListSlowSubscribe,

    // system alarm
    ListSystemAlarm,

    // topic rewrite rule
    ListTopicRewrite,
    CreateTopicRewrite(admin_server::mqtt::topic::CreateTopicRewriteReq),
    DeleteTopicRewrite(admin_server::mqtt::topic::DeleteTopicRewriteReq),

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
    CreateConnector(admin_server::mqtt::connector::CreateConnectorReq),
    DeleteConnector(admin_server::mqtt::connector::DeleteConnectorReq),

    // schema
    ListSchema,
    CreateSchema(admin_server::mqtt::schema::CreateSchemaReq),
    DeleteSchema(admin_server::mqtt::schema::DeleteSchemaReq),
    ListBindSchema,
    BindSchema(admin_server::mqtt::schema::CreateSchemaBindReq),
    UnbindSchema(admin_server::mqtt::schema::DeleteSchemaBindReq),

    //auto subscribe
    ListAutoSubscribe,
    CreateAutoSubscribe(admin_server::mqtt::subscribe::CreateAutoSubscribeReq),
    DeleteAutoSubscribe(admin_server::mqtt::subscribe::DeleteAutoSubscribeReq),
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
            // overview
            MqttActionType::Overview => {
                self.cluster_overview(params.clone()).await;
            }
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
        let request = admin_server::mqtt::session::SessionListReq {
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
            .get_session_list::<admin_server::mqtt::session::SessionListReq, Vec<SessionListRow>>(
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
        cli_request: admin_server::mqtt::user::CreateUserReq,
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
        cli_request: admin_server::mqtt::user::DeleteUserReq,
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
        let request = admin_server::mqtt::user::UserListReq {
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
            .get_user_list::<admin_server::mqtt::user::UserListReq, Vec<admin_server::mqtt::user::UserListRow>>(
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
        cli_request: admin_server::mqtt::acl::CreateAclReq,
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
        cli_request: admin_server::mqtt::acl::DeleteAclReq,
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
        let request = admin_server::mqtt::acl::AclListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_acl_list::<admin_server::mqtt::acl::AclListReq, Vec<admin_server::mqtt::acl::AclListRow>>(
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
        cli_request: admin_server::mqtt::blacklist::CreateBlackListReq,
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
        cli_request: admin_server::mqtt::blacklist::DeleteBlackListReq,
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
        let request = admin_server::mqtt::blacklist::BlackListListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_blacklist::<admin_server::mqtt::blacklist::BlackListListReq, Vec<admin_server::mqtt::blacklist::BlackListListRow>>(
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
        let request = admin_server::mqtt::client::ClientListReq {
            source_ip: None,
            connection_id: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            ..Default::default()
        };

        match admin_client
            .get_client_list::<admin_server::mqtt::client::ClientListReq, Vec<admin_server::mqtt::client::ClientListRow>>(
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
        let request = admin_server::mqtt::system::SystemAlarmListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_flapping_detect_list::<admin_server::mqtt::system::SystemAlarmListReq, Vec<admin_server::mqtt::system::FlappingDetectListRaw>>(
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
        let request = admin_server::mqtt::subscribe::AutoSubscribeListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_slow_subscribe_list::<admin_server::mqtt::subscribe::AutoSubscribeListReq, Vec<admin_server::mqtt::subscribe::SlowSubscribeListRow>>(
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
        let request = admin_server::mqtt::topic::TopicListReq {
            topic_name: None,
            topic_type: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_topic_list::<admin_server::mqtt::topic::TopicListReq, Vec<admin_server::mqtt::topic::TopicListRow>>(
                &request,
            )
            .await
        {
            Ok(page_data) => {
                println!("topic list result:");
                // format table
                let mut table = Table::new();
                table.set_titles(row![
                    "topic_name",
                    "create_time",
                ]);
                for topic in page_data.data {
                    table.add_row(row![
                        topic.topic_name,
                        topic.create_time
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
        let request = admin_server::mqtt::system::SystemAlarmListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_system_alarm_list::<admin_server::mqtt::system::SystemAlarmListReq, Vec<admin_server::mqtt::system::SystemAlarmListRow>>(
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
        let request = admin_server::mqtt::subscribe::SubscribeListReq {
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
            .get_subscribe_list::<admin_server::mqtt::subscribe::SubscribeListReq, Vec<admin_server::mqtt::subscribe::SubscribeListRow>>(
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
        let request = admin_server::mqtt::connector::ConnectorListReq {
            connector_name: None,
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_connector_list::<admin_server::mqtt::connector::ConnectorListReq, Vec<admin_server::mqtt::connector::ConnectorListRow>>(
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
                        connector.topic_name,
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
        cli_request: admin_server::mqtt::connector::CreateConnectorReq,
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
        cli_request: admin_server::mqtt::connector::DeleteConnectorReq,
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
        let request = admin_server::mqtt::topic::TopicRewriteReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_topic_rewrite_list::<admin_server::mqtt::topic::TopicRewriteReq, Vec<admin_server::mqtt::topic::TopicRewriteListRow>>(
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
        cli_request: admin_server::mqtt::topic::CreateTopicRewriteReq,
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
        cli_request: admin_server::mqtt::topic::DeleteTopicRewriteReq,
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
        let request = admin_server::mqtt::schema::SchemaListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_schema_list::<admin_server::mqtt::schema::SchemaListReq, Vec<admin_server::mqtt::schema::SchemaListRow>>(
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
        cli_request: admin_server::mqtt::schema::CreateSchemaReq,
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
        cli_request: admin_server::mqtt::schema::DeleteSchemaReq,
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
        cli_request: admin_server::mqtt::schema::CreateSchemaBindReq,
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
        cli_request: admin_server::mqtt::schema::DeleteSchemaBindReq,
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
        let request = admin_server::mqtt::schema::SchemaBindListReq {
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
            .get_schema_bind_list::<admin_server::mqtt::schema::SchemaBindListReq, Vec<admin_server::mqtt::schema::SchemaBindListRow>>(
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
        cli_request: admin_server::mqtt::subscribe::CreateAutoSubscribeReq,
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
        cli_request: admin_server::mqtt::subscribe::DeleteAutoSubscribeReq,
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
        let request = admin_server::mqtt::subscribe::AutoSubscribeListReq {
            limit: Some(DEFAULT_PAGE_SIZE),
            page: Some(DEFAULT_PAGE_NUM),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_auto_subscribe_list::<admin_server::mqtt::subscribe::AutoSubscribeListReq, Vec<admin_server::mqtt::subscribe::AutoSubscribeListRow>>(
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

    async fn cluster_overview(&self, params: MqttCliCommandParam) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        match admin_client
            .get_cluster_overview::<admin_server::mqtt::overview::OverViewResp>()
            .await
        {
            Ok(overview) => {
                let data = overview.data;
                println!("\n📊 Cluster Overview");
                println!("{:<30} {}", "Cluster Name", data.cluster_name);
                println!("{:<30} {}", "Placement Status", data.placement_status);
                println!("{:<30} {}", "Message In Rate", data.message_in_rate);
                println!("{:<30} {}", "Message Out Rate", data.message_out_rate);
                println!("{:<30} {}", "Connection Num", data.connection_num);
                println!("{:<30} {}", "Session Num", data.session_num);
                println!("{:<30} {}", "Topic Num", data.topic_num);
                println!("{:<30} {}", "Subscribe Num", data.subscribe_num);
                println!("{:<30} {}", "Connector Num", data.connector_num);
                println!("{:<30} {}", "TCP Connections", data.tcp_connection_num);
                println!("{:<30} {}", "TLS Connections", data.tls_connection_num);
                println!(
                    "{:<30} {}",
                    "WebSocket Connections", data.websocket_connection_num
                );
                println!("{:<30} {}", "QUIC Connections", data.quic_connection_num);

                println!("\n🧩 Node List");
                println!(
                    "{:<10} {:<20} {:<25} {:<30} Start Time",
                    "Node ID", "Node IP", "Inner Addr", "Roles"
                );

                use chrono::{Local, TimeZone};
                for node in data.node_list {
                    let start_time_local = Local
                        .timestamp_opt(node.start_time as i64, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| node.start_time.to_string());

                    println!(
                        "{:<10} {:<20} {:<25} {:<30} {}",
                        node.node_id,
                        node.node_ip,
                        node.grpc_addr,
                        format!("{:?}", node.roles),
                        start_time_local
                    );
                }
            }
            Err(e) => {
                println!("MQTT broker cluster overview exception");
                error_info(e.to_string());
            }
        }
    }
}
