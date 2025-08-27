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
use cli_command::mqtt::{MqttActionType, MqttBrokerCommand, MqttCliCommandParam};
use cli_command::placement::{
    PlacementActionType, PlacementCenterCommand, PlacementCliCommandParam,
};
use mqtt::admin::{
    process_auto_subscribe_args, process_config_args, process_connection_args,
    process_session_args, AutoSubscribeRuleCommand, ClusterConfigArgs, ConnectionArgs, SchemaArgs,
    SessionArgs,
};
use mqtt::publish::process_subscribe_args;
use protocol::broker_mqtt::broker_mqtt_admin::EnableFlappingDetectRequest;

use protocol::meta::placement_center_openraft::{AddLearnerRequest, ChangeMembershipRequest, Node};

use crate::mqtt::admin::{
    process_acl_args, process_blacklist_args, process_connector_args, process_schema_args,
    process_slow_sub_args, process_subscribes_args, process_system_alarm_args,
    process_topic_rewrite_args, process_user_args, AclArgs, BlacklistArgs, ConnectorArgs,
    FlappingDetectArgs, SlowSubscribeArgs, SubscribesArgs, SystemAlarmArgs, TopicRewriteArgs,
    UserArgs,
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
    Place(PlacementArgs),
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
    #[arg(short, long,default_value_t =String::from("127.0.0.1:1228"))]
    server: String,

    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
enum MQTTAction {
    // cluster status
    Status,
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
    // flapping detect feat
    FlappingDetect(FlappingDetectArgs),
    // Connections
    Connection(ConnectionArgs),
    // #### observability ####
    // ---- slow subscription ----
    SlowSubscribe(SlowSubscribeArgs),
    // ---- system alarm ----
    SystemAlarm(SystemAlarmArgs),
    // list topic
    ListTopic,
    // topic rewrite rule
    TopicRewriteRule(TopicRewriteArgs),
    // connector
    Connector(ConnectorArgs),

    // schema
    Schema(SchemaArgs),

    //auto subscribe
    AutoSubscribeRule(AutoSubscribeRuleCommand),

    // pub/sub
    Publish(PubSubArgs),
    Subscribe(PubSubArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ",  about="Command line tool for placement center", long_about = None)]
#[command(next_line_help = true)]
struct PlacementArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:1228"))]
    server: String,

    #[clap(subcommand)]
    action: PlacementAction,
}

#[derive(Debug, Subcommand)]
enum PlacementAction {
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

    #[arg(short, long, default_value_t = String::from("127.0.0.1:1228"))]
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
    #[arg(short, long,default_value_t =String::from("127.0.0.1:1228"))]
    server: String,

    #[arg(short, long,default_value_t =String::from("status"))]
    action: String,
}

#[tokio::main]
async fn main() {
    let args = RobustMQCli::parse();
    match args.command {
        RobustMQCliCommand::Mqtt(args) => handle_mqtt(args, MqttBrokerCommand::new()).await,
        RobustMQCliCommand::Place(args) => {
            handle_placement(args, PlacementCenterCommand::new()).await
        }
        RobustMQCliCommand::Journal(args) => handle_journal(args).await,
    }
}

async fn handle_mqtt(args: MqttArgs, cmd: MqttBrokerCommand) {
    let params = MqttCliCommandParam {
        server: args.server,
        action: match args.action {
            // cluster status
            MQTTAction::Status => MqttActionType::Status,
            // cluster status
            MQTTAction::Config(args) => process_config_args(args),
            // session list
            MQTTAction::Session(args) => process_session_args(args),
            // subscribe list
            MQTTAction::Subscribes(args) => process_subscribes_args(args),
            // user admin
            MQTTAction::User(args) => process_user_args(args),
            // access control list admin
            MQTTAction::Acl(args) => process_acl_args(args),
            // blacklist admin
            MQTTAction::Blacklist(args) => process_blacklist_args(args),
            // flapping detect
            MQTTAction::FlappingDetect(args) => {
                MqttActionType::EnableFlappingDetect(EnableFlappingDetectRequest {
                    is_enable: args.is_enable.unwrap_or(false),
                    window_time: args.window_time.unwrap_or(1),
                    max_client_connections: args.max_client_connections.unwrap_or(15),
                    ban_time: args.ban_time.unwrap_or(5),
                })
            }
            // system alarm
            MQTTAction::SystemAlarm(args) => process_system_alarm_args(args),
            // Connections
            MQTTAction::Connection(args) => process_connection_args(args),
            // connector
            MQTTAction::Connector(args) => process_connector_args(args),
            // list topic
            MQTTAction::ListTopic => MqttActionType::ListTopic,
            // topic rewrite rule
            MQTTAction::TopicRewriteRule(args) => process_topic_rewrite_args(args),
            MQTTAction::SlowSubscribe(args) => process_slow_sub_args(args),
            // pub/sub
            MQTTAction::Publish(args) => process_publish_args(args),
            MQTTAction::Subscribe(args) => process_subscribe_args(args),
            // schema
            MQTTAction::Schema(args) => process_schema_args(args),
            // auto subscribe rule
            MQTTAction::AutoSubscribeRule(args) => process_auto_subscribe_args(args),
        },
    };
    cmd.start(params).await;
}

async fn handle_placement(args: PlacementArgs, cmd: PlacementCenterCommand) {
    let params = PlacementCliCommandParam {
        server: args.server,
        action: match args.action {
            PlacementAction::Status => PlacementActionType::Status,
            PlacementAction::AddLearner(arg) => {
                PlacementActionType::AddLearner(AddLearnerRequest {
                    node_id: arg.node_id,
                    node: Some(Node {
                        node_id: arg.node_id,
                        rpc_addr: arg.rpc_addr,
                    }),
                    blocking: arg.blocking,
                })
            }
            PlacementAction::ChangeMembership(arg) => {
                PlacementActionType::ChangeMembership(ChangeMembershipRequest {
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
