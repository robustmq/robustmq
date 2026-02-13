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
use crate::engine::command::{EngineActionType, EngineCliCommandParam, EngineCommand};
use crate::mqtt::command::{MqttBrokerCommand, MqttCliCommandParam};
use crate::mqtt::params::{
    process_acl_args, process_auto_subscribe_args, process_blacklist_args, process_connection_args,
    process_connector_args, process_flapping_detect_args, process_overview, process_publish_args,
    process_schema_args, process_session_args, process_slow_sub_args, process_subscribe_args,
    process_subscribes_args, process_system_alarm_args, process_topic_args,
    process_topic_rewrite_args, process_user_args, AclArgs, AutoSubscribeRuleCommand,
    BlacklistArgs, ClientsArgs, ConnectorArgs, FlappingDetectArgs, PubSubArgs, SchemaArgs,
    SessionArgs, SlowSubscribeArgs, SubscribesArgs, SystemAlarmArgs, TopicArgs, TopicRewriteArgs,
    UserArgs,
};
use crate::output::OutputFormat;
use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Parser)]
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
    Engine(EngineArgs),
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
#[command(author="RobustMQ", about="MQTT management commands", long_about = None)]
#[command(next_line_help = true)]
pub struct MqttArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    server: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 100)]
    limit: u32,
    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
pub enum MQTTAction {
    Overview,
    Session(SessionArgs),
    Subscribes(SubscribesArgs),
    User(UserArgs),
    Acl(AclArgs),
    Blacklist(BlacklistArgs),
    Client(ClientsArgs),
    FlappingDetect(FlappingDetectArgs),
    SlowSubscribe(SlowSubscribeArgs),
    SystemAlarm(SystemAlarmArgs),
    Topic(TopicArgs),
    TopicRewrite(TopicRewriteArgs),
    Connector(ConnectorArgs),
    Schema(SchemaArgs),
    AutoSubscribe(AutoSubscribeRuleCommand),
    Publish(PubSubArgs),
    Subscribe(PubSubArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Cluster management commands", long_about = None)]
#[command(next_line_help = true)]
pub struct ClusterArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    server: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,
    #[clap(subcommand)]
    action: ClusterAction,
}

#[derive(Debug, Subcommand)]
pub enum ClusterAction {
    Status,
    Healthy,
    Config(ClusterConfigArgs),
}

#[derive(clap::Args, Debug)]
pub struct ClusterConfigArgs {
    #[command(subcommand)]
    pub action: ClusterConfigActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum ClusterConfigActionType {
    Get,
    Set(ClusterConfigSetArgs),
}

#[derive(clap::Args, Debug)]
pub struct ClusterConfigSetArgs {
    #[arg(long, required = true)]
    pub config_type: String,
    #[arg(long, required = true)]
    pub config: String,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Storage engine management commands", long_about = None)]
#[command(next_line_help = true)]
pub struct EngineArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    server: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,
    #[arg(long, default_value_t = 1)]
    page: u32,
    #[arg(long, default_value_t = 100)]
    limit: u32,
    #[command(subcommand)]
    action: EngineAction,
}

#[derive(Debug, Subcommand)]
pub enum EngineAction {
    Shard(EngineShardArgs),
    Segment(EngineSegmentArgs),
    Offset(EngineOffsetArgs),
}

#[derive(clap::Args, Debug)]
pub struct EngineShardArgs {
    #[command(subcommand)]
    action: EngineShardAction,
}

#[derive(Debug, Subcommand)]
pub enum EngineShardAction {
    List {
        #[arg(long)]
        shard_name: Option<String>,
    },
    Create {
        #[arg(long, required = true)]
        shard_name: String,
        #[arg(long, required = true)]
        config: String,
    },
    Delete {
        #[arg(long, required = true)]
        shard_name: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct EngineSegmentArgs {
    #[command(subcommand)]
    action: EngineSegmentAction,
}

#[derive(Debug, Subcommand)]
pub enum EngineSegmentAction {
    List {
        #[arg(long, required = true)]
        shard_name: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct EngineOffsetArgs {
    #[command(subcommand)]
    action: EngineOffsetAction,
}

#[derive(Debug, Subcommand)]
pub enum EngineOffsetAction {
    ByTimestamp {
        #[arg(long, required = true)]
        shard_name: String,
        #[arg(long, required = true)]
        timestamp: u64,
        #[arg(long, required = true)]
        strategy: String,
    },
    ByGroup {
        #[arg(long, required = true)]
        group_name: String,
    },
    Commit {
        #[arg(long, required = true)]
        group_name: String,
        #[arg(long, required = true)]
        offsets_json: String,
    },
}

pub async fn handle_mqtt(args: MqttArgs) {
    let params = MqttCliCommandParam {
        server: args.server,
        output: args.output,
        page: args.page,
        limit: args.limit,
        action: match args.action {
            MQTTAction::Overview => process_overview(),
            MQTTAction::Session(args) => process_session_args(args),
            MQTTAction::Subscribes(args) => process_subscribes_args(args),
            MQTTAction::User(args) => process_user_args(args),
            MQTTAction::Acl(args) => match process_acl_args(args) {
                Ok(action) => action,
                Err(e) => {
                    eprintln!("Error processing ACL args: {e}");
                    return;
                }
            },
            MQTTAction::Blacklist(args) => match process_blacklist_args(args) {
                Ok(action) => action,
                Err(e) => {
                    eprintln!("Error processing Blacklist args: {e}");
                    return;
                }
            },
            MQTTAction::FlappingDetect(args) => process_flapping_detect_args(args),
            MQTTAction::SystemAlarm(args) => process_system_alarm_args(args),
            MQTTAction::Client(args) => process_connection_args(args),
            MQTTAction::Connector(args) => process_connector_args(args),
            MQTTAction::Topic(args) => process_topic_args(args),
            MQTTAction::TopicRewrite(args) => process_topic_rewrite_args(args),
            MQTTAction::SlowSubscribe(args) => process_slow_sub_args(args),
            MQTTAction::Publish(args) => process_publish_args(args),
            MQTTAction::Subscribe(args) => process_subscribe_args(args),
            MQTTAction::Schema(args) => process_schema_args(args),
            MQTTAction::AutoSubscribe(args) => process_auto_subscribe_args(args),
        },
    };
    MqttBrokerCommand::new().start(params).await;
}

pub async fn handle_cluster(args: ClusterArgs) {
    let action = match args.action {
        ClusterAction::Status => ClusterActionType::Status,
        ClusterAction::Healthy => ClusterActionType::Healthy,
        ClusterAction::Config(config_args) => match config_args.action {
            ClusterConfigActionType::Get => ClusterActionType::GetConfig,
            ClusterConfigActionType::Set(set_args) => {
                ClusterActionType::SetConfig(admin_server::cluster::ClusterConfigSetReq {
                    config_type: set_args.config_type,
                    config: set_args.config,
                })
            }
        },
    };

    let params = ClusterCliCommandParam {
        server: args.server,
        output: args.output,
        action,
    };
    ClusterCommand::new().start(params).await;
}

pub async fn handle_engine(args: EngineArgs) {
    let action = match args.action {
        EngineAction::Shard(shard_args) => match shard_args.action {
            EngineShardAction::List { shard_name } => EngineActionType::ShardList { shard_name },
            EngineShardAction::Create { shard_name, config } => {
                EngineActionType::ShardCreate { shard_name, config }
            }
            EngineShardAction::Delete { shard_name } => {
                EngineActionType::ShardDelete { shard_name }
            }
        },
        EngineAction::Segment(segment_args) => match segment_args.action {
            EngineSegmentAction::List { shard_name } => {
                EngineActionType::SegmentList { shard_name }
            }
        },
        EngineAction::Offset(offset_args) => match offset_args.action {
            EngineOffsetAction::ByTimestamp {
                shard_name,
                timestamp,
                strategy,
            } => EngineActionType::OffsetByTimestamp {
                shard_name,
                timestamp,
                strategy,
            },
            EngineOffsetAction::ByGroup { group_name } => {
                EngineActionType::OffsetByGroup { group_name }
            }
            EngineOffsetAction::Commit {
                group_name,
                offsets_json,
            } => {
                let offsets: HashMap<String, u64> = match serde_json::from_str(&offsets_json) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!(
                            "Invalid offsets_json, expected object like {{\"shard-a\":1}}: {e}"
                        );
                        return;
                    }
                };
                EngineActionType::CommitOffset {
                    group_name,
                    offsets,
                }
            }
        },
    };

    let params = EngineCliCommandParam {
        server: args.server,
        output: args.output,
        page: args.page,
        limit: args.limit,
        action,
    };
    EngineCommand::new().start(params).await;
}
