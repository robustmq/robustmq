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

use crate::mqtt::command::MqttActionType;
use crate::mqtt::pub_sub::{PublishArgsRequest, SubscribeArgsRequest};
use admin_server::mqtt::connector::FailureStrategy;
use clap::builder::EnumValueParser;
use clap::{arg, Parser};
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use core::option::Option::Some;

// session
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt session, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub action: SessionActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SessionActionType {
    #[command(author = "RobustMQ", about = "action: list sessions", long_about = None)]
    List,
}

// subscribe
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt subscriptions, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct SubscribesArgs {
    #[command(subcommand)]
    pub action: SubscribesActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubscribesActionType {
    #[command(author = "RobustMQ", about = "action: list subscriptions", long_about = None)]
    List,
}

// connection
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt connection, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct ClientsArgs {
    #[command(subcommand)]
    pub action: ClientsActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum ClientsActionType {
    #[command(author = "RobustMQ", about = "action: list clients", long_about = None)]
    List,
}

// cluster config
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt cluster, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct ClusterConfigArgs {
    #[command(subcommand)]
    pub action: ClusterConfigActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum ClusterConfigActionType {
    #[command(author = "RobustMQ", about = "action: get cluster config", long_about = None)]
    Get,
}

// user
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt users, such as listing, creating, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct UserArgs {
    #[command(subcommand)]
    pub action: UserActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum UserActionType {
    #[command(author = "RobustMQ", about = "action: list users", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: create user", long_about = None)]
    Create(CreateUserArgs),
    #[command(author = "RobustMQ", about = "action: delete user", long_about = None)]
    Delete(DeleteUserArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct CreateUserArgs {
    #[arg(short, long, required = true)]
    pub username: String,
    #[arg(short, long, required = true)]
    pub password: String,
    #[arg(short, long, default_value_t = false)]
    pub is_superuser: bool,
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: delete user", long_about = None)]
#[command(next_line_help = true)]
pub struct DeleteUserArgs {
    #[arg(short, long, required = true)]
    pub username: String,
}

// acl feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of access control list, such as listing, creating, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct AclArgs {
    #[command(subcommand)]
    pub action: AclActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum AclActionType {
    #[command(author = "RobustMQ", about = "action: acl list", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: create acl", long_about = None)]
    Create(CreateAclArgs),
    #[command(author = "RobustMQ", about = "action: delete acl", long_about = None)]
    Delete(DeleteAclArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct CreateAclArgs {
    #[arg(short, long, required = true)]
    pub cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclResourceType>::new(),
        default_missing_value = "ClientId"
    )]
    pub resource_type: MqttAclResourceType,
    #[arg(long, required = true)]
    pub resource_name: String,
    #[arg(long, required = true)]
    pub topic: String,
    #[arg(long, required = true)]
    pub ip: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclAction>::new(),
        default_missing_value = "All",
    )]
    pub action: MqttAclAction,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclPermission>::new(),
        default_missing_value = "Allow",
    )]
    pub permission: MqttAclPermission,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct DeleteAclArgs {
    #[arg(short, long, required = true)]
    pub cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclResourceType>::new(),
        default_missing_value = "ClientId"
    )]
    pub resource_type: MqttAclResourceType,
    #[arg(long, required = true)]
    pub resource_name: String,
    #[arg(long, required = true)]
    pub topic: String,
    #[arg(long, required = true)]
    pub ip: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclAction>::new(),
        default_missing_value = "All",
    )]
    pub action: MqttAclAction,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclPermission>::new(),
        default_missing_value = "Allow",
    )]
    pub permission: MqttAclPermission,
}

// blacklist feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of blacklist, such as listing, creating, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct BlacklistArgs {
    #[command(subcommand)]
    pub action: BlackListActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum BlackListActionType {
    #[command(author = "RobustMQ", about = "action: blacklist list", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: create blacklist", long_about = None)]
    Create(CreateBlacklistArgs),
    #[command(author = "RobustMQ", about = "action: delete blacklist", long_about = None)]
    Delete(DeleteBlacklistArgs),
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: create blacklist", long_about = None)]
#[command(next_line_help = true)]
pub struct CreateBlacklistArgs {
    #[arg(short, long, required = true)]
    pub cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclBlackListType>::new(),
        default_missing_value = "ClientId"
    )]
    pub blacklist_type: MqttAclBlackListType,
    #[arg(long, required = true)]
    pub resource_name: String,
    #[arg(long, required = true)]
    pub end_time: u64,
    #[arg(long, required = true)]
    pub desc: String,
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: delete blacklist", long_about = None)]
#[command(next_line_help = true)]
pub struct DeleteBlacklistArgs {
    #[arg(short, long, required = true)]
    pub cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclBlackListType>::new(),
        default_missing_value = "ClientId"
    )]
    pub blacklist_type: MqttAclBlackListType,
    #[arg(short, long, required = true)]
    pub resource_name: String,
}

// #### observability ####
// ---- flapping detect ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: list flapping detect", long_about = None)]
#[command(next_line_help = true)]
pub struct FlappingDetectArgs {}

// ---- slow subscribe ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of slow subscribe, such as listing", long_about = None)]
#[command(next_line_help = true)]
pub struct SlowSubscribeArgs {
    #[command(subcommand)]
    pub action: SlowSubscribeActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SlowSubscribeActionType {
    #[command(author = "RobustMQ", about = "action: list slow subscribe", long_about = None)]
    List,
}

// ---- system alarm ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of topic, such as setting and listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct TopicArgs {
    #[command(subcommand)]
    pub action: TopicActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum TopicActionType {
    #[command(author = "RobustMQ", about = "action: list topic", long_about = None)]
    List,
}

// ---- system alarm ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of system alarm, such as setting and listing", long_about = None
)]
#[command(next_line_help = true)]
pub struct SystemAlarmArgs {
    #[command(subcommand)]
    pub action: SystemAlarmActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SystemAlarmActionType {
    #[command(author = "RobustMQ", about = "action: list system alarm", long_about = None)]
    List,
}

// topic rewrite rule
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of topic rewrite, such as creating and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct TopicRewriteArgs {
    #[command(subcommand)]
    pub action: TopicRewriteActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum TopicRewriteActionType {
    #[command(author = "RobustMQ", about = "action: list topic rewrite rules", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: create topic rewrite", long_about = None)]
    Create(CreateTopicRewriteArgs),
    #[command(author = "RobustMQ", about = "action: delete topic rewrite", long_about = None)]
    Delete(DeleteTopicRewriteArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct CreateTopicRewriteArgs {
    #[arg(short, long, required = true)]
    pub action: String,
    #[arg(short, long, required = true)]
    pub source_topic: String,
    #[arg(short, long, required = true)]
    pub dest_topic: String,
    #[arg(short, long, required = true)]
    pub regex: String,
}
#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct DeleteTopicRewriteArgs {
    #[arg(short, long, required = true)]
    pub action: String,
    #[arg(short, long, required = true)]
    pub source_topic: String,
}

// connector feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of connector, such as listing, creating, updating and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct ConnectorArgs {
    #[command(subcommand)]
    pub action: ConnectorActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConnectorActionType {
    #[command(author = "RobustMQ", about = "action: list connectors", long_about = None)]
    List(ListConnectorArgs),
    #[command(author = "RobustMQ", about = "action: create connector", long_about = None)]
    Create(CreateConnectorArgs),
    #[command(author = "RobustMQ", about = "action: delete connector", long_about = None)]
    Delete(DeleteConnectorArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct CreateConnectorArgs {
    #[arg(short = 'n', long, required = true)]
    pub connector_name: String,
    #[arg(short = 't', long, required = true)]
    pub connector_type: String,
    #[arg(short = 'c', long, required = true)]
    pub config: String,
    #[arg(short = 'p', long, required = true)]
    pub topic_name: String,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct ListConnectorArgs {
    #[arg(short, long, required = true)]
    pub connector_name: String,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct DeleteConnectorArgs {
    #[arg(short, long, required = true)]
    pub connector_name: String,
}

// schema
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt schemas, such as listing, creating, updating, deleting, binding and unbinding", long_about = None
)]
#[command(next_line_help = true)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub action: SchemaActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SchemaActionType {
    #[command(author = "RobustMQ", about = "action: list schemas", long_about = None)]
    List(ListSchemaArgs),
    #[command(author = "RobustMQ", about = "action: create schema", long_about = None)]
    Create(CreateSchemaArgs),
    #[command(author = "RobustMQ", about = "action: delete schema", long_about = None)]
    Delete(DeleteSchemaArgs),
    #[command(author = "RobustMQ", about = "action: list bind schemas", long_about = None)]
    ListBind(ListBindSchemaArgs),
    #[command(author = "RobustMQ", about = "action: bind schema", long_about = None)]
    Bind(BindSchemaArgs),
    #[command(author = "RobustMQ", about = "action: unbind schema", long_about = None)]
    Unbind(UnbindSchemaArgs),
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: list schema", long_about = None)]
#[command(next_line_help = true)]
pub struct ListSchemaArgs {
    #[arg(short, long, required = true)]
    pub schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: create schema", long_about = None)]
#[command(next_line_help = true)]
pub struct CreateSchemaArgs {
    #[arg(short = 'n', long, required = true)]
    pub schema_name: String,
    #[arg(short = 't', long, required = true)]
    pub schema_type: String,
    #[arg(short = 's', long, required = true)]
    pub schema: String,
    #[arg(short = 'd', long, required = true)]
    pub desc: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: delete schema", long_about = None)]
#[command(next_line_help = true)]
pub struct DeleteSchemaArgs {
    #[arg(short, long, required = true)]
    pub schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: list bind schema", long_about = None)]
#[command(next_line_help = true)]
pub struct ListBindSchemaArgs {
    #[arg(short, long, required = true)]
    pub schema_name: String,
    #[arg(short, long, required = true)]
    pub resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: bind schema", long_about = None)]
#[command(next_line_help = true)]
pub struct BindSchemaArgs {
    #[arg(short, long, required = true)]
    pub schema_name: String,
    #[arg(short, long, required = true)]
    pub resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: unbind schema", long_about = None)]
#[command(next_line_help = true)]
pub struct UnbindSchemaArgs {
    #[arg(short, long, required = true)]
    pub schema_name: String,
    #[arg(short, long, required = true)]
    pub resource_name: String,
}

pub fn process_slow_sub_args(args: SlowSubscribeArgs) -> MqttActionType {
    match args.action {
        SlowSubscribeActionType::List => MqttActionType::ListSlowSubscribe,
    }
}

pub fn process_flapping_detect_args(_args: FlappingDetectArgs) -> MqttActionType {
    MqttActionType::ListFlappingDetect
}

pub fn process_system_alarm_args(args: SystemAlarmArgs) -> MqttActionType {
    match args.action {
        SystemAlarmActionType::List => MqttActionType::ListSystemAlarm,
    }
}

pub fn process_session_args(args: SessionArgs) -> MqttActionType {
    match args.action {
        SessionActionType::List => MqttActionType::ListSession,
    }
}

pub fn process_subscribes_args(args: SubscribesArgs) -> MqttActionType {
    match args.action {
        SubscribesActionType::List => MqttActionType::ListSubscribe,
    }
}

pub fn process_user_args(args: UserArgs) -> MqttActionType {
    match args.action {
        UserActionType::List => MqttActionType::ListUser,
        UserActionType::Create(arg) => {
            MqttActionType::CreateUser(admin_server::mqtt::user::CreateUserReq {
                username: arg.username,
                password: arg.password,
                is_superuser: arg.is_superuser,
            })
        }
        UserActionType::Delete(arg) => {
            MqttActionType::DeleteUser(admin_server::mqtt::user::DeleteUserReq {
                username: arg.username,
            })
        }
    }
}

pub fn process_acl_args(args: AclArgs) -> Result<MqttActionType, Box<dyn std::error::Error>> {
    match args.action {
        AclActionType::List => Ok(MqttActionType::ListAcl),
        AclActionType::Create(arg) => Ok(MqttActionType::CreateAcl(
            admin_server::mqtt::acl::CreateAclReq {
                resource_type: arg.resource_type.to_string(),
                resource_name: arg.resource_name,
                topic: arg.topic,
                ip: arg.ip,
                action: arg.action.to_string(),
                permission: arg.permission.to_string(),
            },
        )),
        AclActionType::Delete(arg) => Ok(MqttActionType::DeleteAcl(
            admin_server::mqtt::acl::DeleteAclReq {
                resource_type: arg.resource_type.to_string(),
                resource_name: arg.resource_name,
                topic: arg.topic,
                ip: arg.ip,
                action: arg.action.to_string(),
                permission: arg.permission.to_string(),
            },
        )),
    }
}

pub fn process_blacklist_args(
    args: BlacklistArgs,
) -> Result<MqttActionType, Box<dyn std::error::Error>> {
    match args.action {
        BlackListActionType::List => Ok(MqttActionType::ListBlacklist),
        BlackListActionType::Create(arg) => Ok(MqttActionType::CreateBlacklist(
            admin_server::mqtt::blacklist::CreateBlackListReq {
                blacklist_type: arg.blacklist_type.to_string(),
                resource_name: arg.resource_name,
                end_time: arg.end_time,
                desc: arg.desc,
            },
        )),
        BlackListActionType::Delete(arg) => Ok(MqttActionType::DeleteBlacklist(
            admin_server::mqtt::blacklist::DeleteBlackListReq {
                blacklist_type: arg.blacklist_type.to_string(),
                resource_name: arg.resource_name,
            },
        )),
    }
}

pub fn process_connection_args(args: ClientsArgs) -> MqttActionType {
    match args.action {
        ClientsActionType::List => MqttActionType::ListClient,
    }
}

pub fn process_topic_args(args: TopicArgs) -> MqttActionType {
    match args.action {
        TopicActionType::List => MqttActionType::ListTopic,
    }
}

pub fn process_connector_args(args: ConnectorArgs) -> MqttActionType {
    match args.action {
        ConnectorActionType::List(_) => MqttActionType::ListConnector,
        ConnectorActionType::Create(arg) => {
            MqttActionType::CreateConnector(admin_server::mqtt::connector::CreateConnectorReq {
                connector_name: arg.connector_name,
                connector_type: arg.connector_type,
                config: arg.config,
                failure_strategy: FailureStrategy {
                    strategy: "discard".to_string(),
                    ..Default::default()
                },
                topic_name: arg.topic_name,
            })
        }
        ConnectorActionType::Delete(arg) => {
            MqttActionType::DeleteConnector(admin_server::mqtt::connector::DeleteConnectorReq {
                connector_name: arg.connector_name,
            })
        }
    }
}

pub fn process_topic_rewrite_args(args: TopicRewriteArgs) -> MqttActionType {
    match args.action {
        TopicRewriteActionType::List => MqttActionType::ListTopicRewrite,
        TopicRewriteActionType::Create(arg) => {
            MqttActionType::CreateTopicRewrite(admin_server::mqtt::topic::CreateTopicRewriteReq {
                action: arg.action,
                source_topic: arg.source_topic,
                dest_topic: arg.dest_topic,
                regex: arg.regex,
            })
        }
        TopicRewriteActionType::Delete(arg) => {
            MqttActionType::DeleteTopicRewrite(admin_server::mqtt::topic::DeleteTopicRewriteReq {
                action: arg.action,
                source_topic: arg.source_topic,
            })
        }
    }
}

pub fn process_schema_args(args: SchemaArgs) -> MqttActionType {
    match args.action {
        SchemaActionType::Create(arg) => {
            MqttActionType::CreateSchema(admin_server::mqtt::schema::CreateSchemaReq {
                schema_name: arg.schema_name,
                schema_type: arg.schema_type,
                schema: arg.schema,
                desc: arg.desc,
            })
        }
        SchemaActionType::List(_) => MqttActionType::ListSchema,
        SchemaActionType::Delete(arg) => {
            MqttActionType::DeleteSchema(admin_server::mqtt::schema::DeleteSchemaReq {
                schema_name: arg.schema_name,
            })
        }
        SchemaActionType::ListBind(_) => MqttActionType::ListBindSchema,
        SchemaActionType::Bind(arg) => {
            MqttActionType::BindSchema(admin_server::mqtt::schema::CreateSchemaBindReq {
                schema_name: arg.schema_name,
                resource_name: arg.resource_name,
            })
        }
        SchemaActionType::Unbind(arg) => {
            MqttActionType::UnbindSchema(admin_server::mqtt::schema::DeleteSchemaBindReq {
                schema_name: arg.schema_name,
                resource_name: arg.resource_name,
            })
        }
    }
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt auto subscribe, such as listing, setting, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub struct AutoSubscribeRuleCommand {
    #[command(subcommand)]
    pub action: AutoSubscribeRuleActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum AutoSubscribeRuleActionType {
    #[command(author = "RobustMQ", about = "action: auto subscribe rule list", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: delete auto subscribe rule", long_about = None)]
    Delete(DeleteAutoSubscribeRuleArgs),
    #[command(author = "RobustMQ", about = "action: create auto subscribe rule", long_about = None)]
    Create(SetAutoSubscribeRuleArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct SetAutoSubscribeRuleArgs {
    #[arg(short, long, required = true)]
    pub topic: String,
    #[arg(short, long, default_value_t = 0)]
    pub qos: u8,
    #[arg(short, long, default_value_t = false)]
    pub no_local: bool,
    #[arg(short = 'r', long, default_value_t = false)]
    pub retain_as_published: bool,
    #[arg(short = 'R', long, default_value_t = 0)]
    pub retained_handling: u8,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub struct DeleteAutoSubscribeRuleArgs {
    #[arg(short, long, required = true)]
    pub topic: String,
}

pub fn process_auto_subscribe_args(args: AutoSubscribeRuleCommand) -> MqttActionType {
    match args.action {
        AutoSubscribeRuleActionType::List => MqttActionType::ListAutoSubscribe,
        AutoSubscribeRuleActionType::Create(arg) => MqttActionType::CreateAutoSubscribe(
            admin_server::mqtt::subscribe::CreateAutoSubscribeReq {
                topic: arg.topic,
                qos: arg.qos as u32,
                no_local: arg.no_local,
                retain_as_published: arg.retain_as_published,
                retained_handling: arg.retained_handling as u32,
            },
        ),
        AutoSubscribeRuleActionType::Delete(arg) => MqttActionType::DeleteAutoSubscribe(
            admin_server::mqtt::subscribe::DeleteAutoSubscribeReq {
                topic_name: arg.topic,
            },
        ),
    }
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="Command line tool for mqtt broker", long_about = None)]
#[command(next_line_help = true)]
pub struct PubSubArgs {
    #[arg(help = "user name")]
    #[arg(short, long, required = true)]
    username: String,
    #[arg(help = "password")]
    #[arg(short, long, required = true)]
    password: String,
    #[arg(help = "topic")]
    #[arg(short, long, required = true)]
    topic: String,
    #[arg(help = "qos")]
    #[arg(short, long, default_value_t = 0)]
    qos: i32,
    #[arg(long = "retained")]
    #[arg(required = false, default_value_t = false)]
    #[arg(help = "Enable or disable the feature , If it is a subscription, ignore this item")]
    retained: bool,
}

pub fn process_publish_args(args: PubSubArgs) -> MqttActionType {
    MqttActionType::Publish(PublishArgsRequest {
        username: args.username,
        password: args.password,
        topic: args.topic,
        qos: args.qos,
        retained: args.retained,
    })
}

pub fn process_subscribe_args(args: PubSubArgs) -> MqttActionType {
    MqttActionType::Subscribe(SubscribeArgsRequest {
        username: args.username,
        password: args.password,
        topic: args.topic,
        qos: args.qos,
    })
}
