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

use crate::cluster::command::{ClusterActionType, ClusterCliCommandParam, ClusterCommand};
use crate::mqtt::command::{MqttBrokerCommand, MqttCliCommandParam};
use crate::mqtt::params::{
    process_acl_args, process_auto_subscribe_args, process_blacklist_args, process_connection_args,
    process_connector_args, process_flapping_detect_args, process_publish_args,
    process_schema_args, process_session_args, process_slow_sub_args, process_subscribe_args,
    process_subscribes_args, process_system_alarm_args, process_topic_args,
    process_topic_rewrite_args, process_user_args, AclArgs, AutoSubscribeRuleCommand,
    BlacklistArgs, ClientsArgs, ClusterConfigActionType, ClusterConfigArgs, ConnectorArgs,
    FlappingDetectArgs, PubSubArgs, SchemaArgs, SessionArgs, SlowSubscribeArgs, SubscribesArgs,
    SystemAlarmArgs, TopicArgs, TopicRewriteArgs, UserArgs,
};
use clap::{arg, Parser, Subcommand};

#[derive(Parser)] // requires `derive` feature
#[command(name = "robust-ctl")]
#[command(bin_name = "robust-ctl")]
#[command(styles = CLAP_STYLING)]
#[command(author="RobustMQ", version="0.0.1", about="Command line tool for RobustMQ", long_about = None)]
#[command(next_line_help = true)]
pub struct RobustMQCli {
    #[command(subcommand)]
    pub command: RobustMQCliCommand,
}

#[derive(Debug, Subcommand)]
pub enum RobustMQCliCommand {
    Mqtt(MqttArgs),
    Cluster(ClusterArgs),
    Journal(JournalArgs),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Command line tool for mqtt broker", long_about = None)]
#[command(next_line_help = true)]
pub struct MqttArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:8080"))]
    server: String,

    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
pub enum MQTTAction {
    // session admin
    Session(SessionArgs),
    // session admin
    Subscribes(SubscribesArgs),
    // user admin
    User(UserArgs),
    // access control list admin
    Acl(AclArgs),
    // blacklist admin
    Blacklist(BlacklistArgs),
    // Clients
    Client(ClientsArgs),
    // #### observability ####
    // ---- flapping detect ----
    FlappingDetect(FlappingDetectArgs),
    // ---- slow subscription ----
    SlowSubscribe(SlowSubscribeArgs),
    // ---- system alarm ----
    SystemAlarm(SystemAlarmArgs),

    // list topic
    Topic(TopicArgs),

    // topic rewrite
    TopicRewrite(TopicRewriteArgs),

    // connector
    Connector(ConnectorArgs),

    // schema
    Schema(SchemaArgs),

    //auto subscribe
    AutoSubscribe(AutoSubscribeRuleCommand),

    // pub/sub
    Publish(PubSubArgs),
    Subscribe(PubSubArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ",  about="Command line tool for placement center", long_about = None)]
#[command(next_line_help = true)]
pub struct ClusterArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    server: String,

    #[clap(subcommand)]
    action: ClusterAction,
}

#[derive(Debug, Subcommand)]
pub enum ClusterAction {
    Config(ClusterConfigArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Command line tool for journal engine", long_about = None)]
#[command(next_line_help = true)]
pub struct JournalArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:8080"))]
    server: String,

    #[arg(short, long,default_value_t =String::from("status"))]
    action: String,
}

pub async fn handle_mqtt(args: MqttArgs, cmd: MqttBrokerCommand) {
    let params = MqttCliCommandParam {
        server: args.server,
        action: match args.action {
            // session list
            MQTTAction::Session(args) => process_session_args(args),
            // subscribe list
            MQTTAction::Subscribes(args) => process_subscribes_args(args),
            // user admin
            MQTTAction::User(args) => process_user_args(args),
            // access control list admin
            MQTTAction::Acl(args) => match process_acl_args(args) {
                Ok(action) => action,
                Err(e) => {
                    eprintln!("Error processing ACL args: {e}");
                    return;
                }
            },
            // blacklist admin
            MQTTAction::Blacklist(args) => match process_blacklist_args(args) {
                Ok(action) => action,
                Err(e) => {
                    eprintln!("Error processing Blacklist args: {e}");
                    return;
                }
            },
            // flapping detect
            MQTTAction::FlappingDetect(args) => process_flapping_detect_args(args),
            // system alarm
            MQTTAction::SystemAlarm(args) => process_system_alarm_args(args),
            // Connections
            MQTTAction::Client(args) => process_connection_args(args),
            // connector
            MQTTAction::Connector(args) => process_connector_args(args),
            // list topic
            MQTTAction::Topic(args) => process_topic_args(args),
            // topic rewrite rule
            MQTTAction::TopicRewrite(args) => process_topic_rewrite_args(args),
            MQTTAction::SlowSubscribe(args) => process_slow_sub_args(args),
            // pub/sub
            MQTTAction::Publish(args) => process_publish_args(args),
            MQTTAction::Subscribe(args) => process_subscribe_args(args),
            // schema
            MQTTAction::Schema(args) => process_schema_args(args),
            // auto subscribe rule
            MQTTAction::AutoSubscribe(args) => process_auto_subscribe_args(args),
        },
    };
    cmd.start(params).await;
}

pub async fn handle_cluster(args: ClusterArgs, cmd: ClusterCommand) {
    let params = ClusterCliCommandParam {
        server: args.server,
        action: match args.action {
            ClusterAction::Config(config_args) => match config_args.action {
                ClusterConfigActionType::Get => ClusterActionType::GetConfig,
            },
        },
    };
    cmd.start(params).await;
}

// TODO: implement journal engine
pub async fn handle_journal(args: JournalArgs) {
    println!("{args:?}");
}
