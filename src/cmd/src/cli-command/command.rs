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

pub(crate) mod mqtt;

use clap::{arg, Parser, Subcommand};
use cli_command::cluster::{ClusterActionType, ClusterCliCommandParam, ClusterCommand};
use cli_command::mqtt::{MqttBrokerCommand, MqttCliCommandParam};
use mqtt::admin::{
    process_auto_subscribe_args, process_config_args, process_connection_args,
    process_session_args, AutoSubscribeRuleCommand, ClientsArgs, ClusterConfigArgs, SchemaArgs,
    SessionArgs,
};
use mqtt::publish::process_subscribe_args;

use protocol::meta::placement_center_openraft::{AddLearnerRequest, ChangeMembershipRequest, Node};

use crate::mqtt::admin::{
    process_acl_args, process_blacklist_args, process_connector_args, process_flapping_detect_args,
    process_schema_args, process_slow_sub_args, process_subscribes_args, process_system_alarm_args,
    process_topic_args, process_topic_rewrite_args, process_user_args, AclArgs, BlacklistArgs,
    ConnectorArgs, FlappingDetectArgs, SlowSubscribeArgs, SubscribesArgs, SystemAlarmArgs,
    TopicArgs, TopicRewriteArgs, UserArgs,
};
use crate::mqtt::publish::{process_publish_args, PubSubArgs};

#[derive(Parser)] // requires `derive` feature
#[command(name = "robust-ctl")]
#[command(bin_name = "robust-ctl")]
#[command(styles = CLAP_STYLING)]
#[command(author="RobustMQ", version="0.0.1", about="Command line tool for RobustMQ", long_about = None)]
#[command(next_line_help = true)]
struct RobustMQCli {
    #[command(subcommand)]
    command: RobustMQCliCommand,
}

#[derive(Debug, Subcommand)]
enum RobustMQCliCommand {
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
struct MqttArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:8080"))]
    server: String,

    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
enum MQTTAction {
    // session admin
    Config(ClusterConfigArgs),
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
struct ClusterArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    server: String,

    #[clap(subcommand)]
    action: ClusterAction,
}

#[derive(Debug, Subcommand)]
enum ClusterAction {
    Status,
    AddLearner(AddLearnerArgs),
    ChangeMembership(ChangeMembershipArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: add learner", long_about = None)]
#[command(next_line_help = true)]
struct AddLearnerArgs {
    #[arg(short, long, required = true)]
    node_id: u64,

    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    rpc_addr: String,

    #[arg(short, long, default_value_t = true)]
    blocking: bool,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ",  about="action: change membership", long_about = None)]
#[command(next_line_help = true)]
struct ChangeMembershipArgs {
    #[arg(short, long, num_args = 1.., required = true)]
    members: Vec<u64>,

    #[arg(short, long, default_value_t = true)]
    retain: bool,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Command line tool for journal engine", long_about = None)]
#[command(next_line_help = true)]
struct JournalArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:8080"))]
    server: String,

    #[arg(short, long,default_value_t =String::from("status"))]
    action: String,
}

#[tokio::main]
async fn main() {
    let args = RobustMQCli::parse();
    match args.command {
        RobustMQCliCommand::Mqtt(args) => handle_mqtt(args, MqttBrokerCommand::new()).await,
        RobustMQCliCommand::Cluster(args) => handle_placement(args, ClusterCommand::new()).await,
        RobustMQCliCommand::Journal(args) => handle_journal(args).await,
    }
}

async fn handle_mqtt(args: MqttArgs, cmd: MqttBrokerCommand) {
    let params = MqttCliCommandParam {
        server: args.server,
        action: match args.action {
            // cluster status
            MQTTAction::Config(args) => process_config_args(args),
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

async fn handle_placement(args: ClusterArgs, cmd: ClusterCommand) {
    let params = ClusterCliCommandParam {
        server: args.server,
        action: match args.action {
            ClusterAction::Status => ClusterActionType::Status,
            ClusterAction::AddLearner(arg) => ClusterActionType::AddLearner(AddLearnerRequest {
                node_id: arg.node_id,
                node: Some(Node {
                    node_id: arg.node_id,
                    rpc_addr: arg.rpc_addr,
                }),
                blocking: arg.blocking,
            }),
            ClusterAction::ChangeMembership(arg) => {
                ClusterActionType::ChangeMembership(ChangeMembershipRequest {
                    members: arg.members,
                    retain: arg.retain,
                })
            }
        },
    };
    cmd.start(params).await;
}

// TODO: implement journal engine
async fn handle_journal(args: JournalArgs) {
    println!("{args:?}");
}
