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

use clap::builder::{
    ArgAction, BoolishValueParser, EnumValueParser, NonEmptyStringValueParser, RangedU64ValueParser,
};
use clap::{arg, Parser};
use cli_command::mqtt::MqttActionType;
use common_base::enum_type::feature_type::FeatureType;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::enum_type::sort_type::SortType;

use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
use core::option::Option::Some;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use protocol::broker::broker_mqtt_admin::{
    BindSchemaRequest, CreateAclRequest, CreateBlacklistRequest, CreateConnectorRequest,
    CreateSchemaRequest, CreateTopicRewriteRuleRequest, CreateUserRequest, DeleteAclRequest,
    DeleteAutoSubscribeRuleRequest, DeleteBlacklistRequest, DeleteConnectorRequest,
    DeleteSchemaRequest, DeleteTopicRewriteRuleRequest, DeleteUserRequest,
    ListAutoSubscribeRuleRequest, ListBindSchemaRequest, ListConnectorRequest, ListSchemaRequest,
    ListSlowSubscribeRequest, ListSubscribeRequest, ListSystemAlarmRequest,
    SetAutoSubscribeRuleRequest, SetClusterConfigRequest, SetSlowSubscribeConfigRequest,
    SetSystemAlarmConfigRequest, SubscribeDetailRequest, UnbindSchemaRequest,
    UpdateConnectorRequest, UpdateSchemaRequest,
};

// session
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt session, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct SessionArgs {
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
pub(crate) struct SubscribesArgs {
    #[command(subcommand)]
    pub action: SubscribesActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubscribesActionType {
    #[command(author = "RobustMQ", about = "action: list subscriptions", long_about = None)]
    List,

    #[command(author = "RobustMQ", about = "action: detail subscription", long_about = None)]
    Detail(DetailSubscribeArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct DetailSubscribeArgs {
    #[arg(short, long, required = true)]
    pub(crate) client_id: String,
    #[arg(short, long, required = true)]
    pub(crate) path: String,
}

// connection
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt connection, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct ConnectionArgs {
    #[command(subcommand)]
    pub action: ConnectionActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConnectionActionType {
    #[command(author = "RobustMQ", about = "action: list connection", long_about = None)]
    List,
}

// cluster config
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt cluster, such as listing", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct ClusterConfigArgs {
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
pub(crate) struct UserArgs {
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
pub(crate) struct CreateUserArgs {
    #[arg(short, long, required = true)]
    pub(crate) username: String,
    #[arg(short, long, required = true)]
    pub(crate) password: String,
    #[arg(short, long, default_value_t = false)]
    pub(crate) is_superuser: bool,
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: delete user", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteUserArgs {
    #[arg(short, long, required = true)]
    pub(crate) username: String,
}

// acl feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of access control list, such as listing, creating, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct AclArgs {
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
pub(crate) struct CreateAclArgs {
    #[arg(short, long, required = true)]
    pub(crate) cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclResourceType>::new(),
        default_missing_value = "ClientId"
    )]
    pub(crate) resource_type: MqttAclResourceType,
    #[arg(long, required = true)]
    pub(crate) resource_name: String,
    #[arg(long, required = true)]
    pub(crate) topic: String,
    #[arg(long, required = true)]
    pub(crate) ip: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclAction>::new(),
        default_missing_value = "All",
    )]
    pub(crate) action: MqttAclAction,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclPermission>::new(),
        default_missing_value = "Allow",
    )]
    pub(crate) permission: MqttAclPermission,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct DeleteAclArgs {
    #[arg(short, long, required = true)]
    pub(crate) cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclResourceType>::new(),
        default_missing_value = "ClientId"
    )]
    pub(crate) resource_type: MqttAclResourceType,
    #[arg(long, required = true)]
    pub(crate) resource_name: String,
    #[arg(long, required = true)]
    pub(crate) topic: String,
    #[arg(long, required = true)]
    pub(crate) ip: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclAction>::new(),
        default_missing_value = "All",
    )]
    pub(crate) action: MqttAclAction,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclPermission>::new(),
        default_missing_value = "Allow",
    )]
    pub(crate) permission: MqttAclPermission,
}

// blacklist feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of blacklist, such as listing, creating, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct BlacklistArgs {
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
pub(crate) struct CreateBlacklistArgs {
    #[arg(short, long, required = true)]
    pub(crate) cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclBlackListType>::new(),
        default_missing_value = "ClientId"
    )]
    pub(crate) blacklist_type: MqttAclBlackListType,
    #[arg(long, required = true)]
    pub(crate) resource_name: String,
    #[arg(long, required = true)]
    pub(crate) end_time: u64,
    #[arg(long, required = true)]
    pub(crate) desc: String,
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: delete blacklist", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteBlacklistArgs {
    #[arg(short, long, required = true)]
    pub(crate) cluster_name: String,
    #[arg(
        long,
        value_parser = EnumValueParser::<MqttAclBlackListType>::new(),
        default_missing_value = "ClientId"
    )]
    pub(crate) blacklist_type: MqttAclBlackListType,
    #[arg(short, long, required = true)]
    pub(crate) resource_name: String,
}

// flapping detect feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: flapping detect configuration", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct FlappingDetectArgs {
    #[arg(
        long = "enable",
        value_parser = BoolishValueParser::new(),
        default_missing_value = "false",
        action = ArgAction::Set,
        num_args = 0..=1,
        required = true,
        require_equals = true,
        exclusive = true,
        help = "Enable or disable the feature"
    )]
    pub(crate) is_enable: Option<bool>,
    #[arg(
        long = "window-time",
        value_parser = RangedU64ValueParser::<u32>::new(),
        default_missing_value = "1",
        action = ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        help = "Unit is minutes"
    )]
    pub(crate) window_time: Option<u32>,
    #[arg(
        long = "max-client-connections",
        value_parser = RangedU64ValueParser::<u32>::new(),
        default_missing_value = "15",
        action = ArgAction::Set,
        num_args = 0..=1,
        help = "Max client connections"
    )]
    pub(crate) max_client_connections: Option<u32>,
    #[arg(
        long = "ban-time",
        value_parser = RangedU64ValueParser::<u32>::new(),
        default_missing_value = "5",
        action = ArgAction::Set,
        num_args = 0..=1,
        help = "Unit is minutes"
    )]
    pub(crate) ban_time: Option<u32>,
}

// #### observability ####
// ---- slow subscribe ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "action: slow subscribe configuration", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct SlowSubscribeArgs {
    #[command(subcommand)]
    pub action: SlowSubscribeActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SlowSubscribeActionType {
    #[command(author = "RobustMQ", about = "action: enable or disable slow subscribe", long_about = None
    )]
    Enable(EnableSlowSubscribeArgs),
    #[command(author = "RobustMQ", about = "action: set slow subscribe configuration", long_about = None
    )]
    Set(SetSlowSubscribeArgs),
    #[command(author = "RobustMQ", about = "action: list slow subscribe", long_about = None)]
    List(ListSlowSubscribeArgs),
}

#[derive(Debug, clap::Args)]
#[command(next_line_help = true)]
pub(crate) struct EnableSlowSubscribeArgs {
    #[arg(long, required = true)]
    pub(crate) is_enable: Option<bool>,
}

#[derive(Debug, clap::Args)]
#[command(next_line_help = true)]
pub(crate) struct SetSlowSubscribeArgs {
    #[arg(long, default_value_t = 1000, required = false)]
    pub(crate) max_store_num: u32,
    #[arg(long, required = true)]
    pub(crate) delay_type: String,
}

#[derive(Debug, clap::Args)]
#[command(next_line_help = true)]
pub(crate) struct ListSlowSubscribeArgs {
    #[arg(
        long = "list",
        value_parser = RangedU64ValueParser::<u64>::new(),
        default_missing_value = "100",
        help = "List the slow subscriptions"
    )]
    pub(crate) list: Option<u64>,

    #[arg(
        long = "sort",
        required = false,
        require_equals = true,
        value_parser = EnumValueParser::<SortType>::new(),
        default_missing_value = "DESC",
        ignore_case = true,
        help = "Sort the results"
    )]
    pub(crate) sort: Option<SortType>,

    #[arg(
        long = "topic",
        required = false,
        require_equals = true,
        value_parser = NonEmptyStringValueParser::new(),
        help = "Filter the results by topic"
    )]
    pub(crate) topic: Option<String>,

    #[arg(
        long = "sub-name",
        required = false,
        require_equals = true,
        value_parser = NonEmptyStringValueParser::new(),
        help = "Filter the results by subscription name"
    )]
    pub(crate) sub_name: Option<String>,

    #[arg(
        long = "client-id",
        required = false,
        require_equals = true,
        value_parser = NonEmptyStringValueParser::new(),
        help = "Filter the results by client ID"
    )]
    pub(crate) client_id: Option<String>,
}

// ---- system alarm ----
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of system alarm, such as setting and listing", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct SystemAlarmArgs {
    #[command(subcommand)]
    pub action: SystemAlarmActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SystemAlarmActionType {
    #[command(author = "RobustMQ", about = "action: set system alarm", long_about = None)]
    Set(SetSystemAlarmArgs),
    #[command(author = "RobustMQ", about = "action: list system alarm", long_about = None)]
    List,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct SetSystemAlarmArgs {
    #[arg(long, required = false)]
    pub(crate) enable: Option<bool>,
    #[arg(long, required = false)]
    pub(crate) cpu_high_watermark: Option<f32>,
    #[arg(long, required = false)]
    pub(crate) cpu_low_watermark: Option<f32>,
    #[arg(long, required = false)]
    pub(crate) memory_high_watermark: Option<f32>,
    #[arg(long, required = false)]
    pub(crate) os_cpu_check_interval_ms: Option<u64>,
}

// topic rewrite rule
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of topic rewrite, such as creating and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct TopicRewriteArgs {
    #[command(subcommand)]
    pub action: TopicRewriteActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum TopicRewriteActionType {
    #[command(author = "RobustMQ", about = "action: create topic rewrite", long_about = None)]
    Create(CreateTopicRewriteArgs),
    #[command(author = "RobustMQ", about = "action: delete topic rewrite", long_about = None)]
    Delete(DeleteTopicRewriteArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct CreateTopicRewriteArgs {
    #[arg(short, long, required = true)]
    pub(crate) action: String,
    #[arg(short, long, required = true)]
    pub(crate) source_topic: String,
    #[arg(short, long, required = true)]
    pub(crate) dest_topic: String,
    #[arg(short, long, required = true)]
    pub(crate) regex: String,
}
#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct DeleteTopicRewriteArgs {
    #[arg(short, long, required = true)]
    pub(crate) action: String,
    #[arg(short, long, required = true)]
    pub(crate) source_topic: String,
}

// connector feat
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of connector, such as listing, creating, updating and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct ConnectorArgs {
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
    #[command(author = "RobustMQ", about = "action: update connector", long_about = None)]
    Update(UpdateConnectorArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct CreateConnectorArgs {
    #[arg(short, long, required = true)]
    pub(crate) connector_name: String,
    #[arg(short, long, required = true)]
    pub(crate) connector_type: String,
    #[arg(short, long, required = true)]
    pub(crate) config: String,
    #[arg(short, long, required = true)]
    pub(crate) topic_id: String,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct ListConnectorArgs {
    #[arg(short, long, required = true)]
    pub(crate) connector_name: String,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct DeleteConnectorArgs {
    #[arg(short, long, required = true)]
    pub(crate) connector_name: String,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct UpdateConnectorArgs {
    #[arg(short, long, required = true)]
    pub(crate) connector: String,
}

// schema
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt schemas, such as listing, creating, updating, deleting, binding and unbinding", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct SchemaArgs {
    #[command(subcommand)]
    pub action: SchemaActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum SchemaActionType {
    #[command(author = "RobustMQ", about = "action: list schemas", long_about = None)]
    List(ListSchemaArgs),
    #[command(author = "RobustMQ", about = "action: create schema", long_about = None)]
    Create(CreateSchemaArgs),
    #[command(author = "RobustMQ", about = "action: update schema", long_about = None)]
    Update(UpdateSchemaArgs),
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
pub(crate) struct ListSchemaArgs {
    #[arg(short, long, required = true)]
    pub(crate) schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: create schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct CreateSchemaArgs {
    #[arg(short = 'n', long, required = true)]
    pub(crate) schema_name: String,
    #[arg(short = 't', long, required = true)]
    pub(crate) schema_type: String,
    #[arg(short = 's', long, required = true)]
    pub(crate) schema: String,
    #[arg(short = 'd', long, required = true)]
    pub(crate) desc: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: update schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UpdateSchemaArgs {
    #[arg(short = 'n', long, required = true)]
    pub(crate) schema_name: String,
    #[arg(short = 't', long, required = true)]
    pub(crate) schema_type: String,
    #[arg(short = 's', long, required = true)]
    pub(crate) schema: String,
    #[arg(short = 'd', long, required = true)]
    pub(crate) desc: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: delete schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteSchemaArgs {
    #[arg(short, long, required = true)]
    pub(crate) schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: list bind schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct ListBindSchemaArgs {
    #[arg(short, long, required = true)]
    pub(crate) schema_name: String,
    #[arg(short, long, required = true)]
    pub(crate) resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: bind schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct BindSchemaArgs {
    #[arg(short, long, required = true)]
    pub(crate) schema_name: String,
    #[arg(short, long, required = true)]
    pub(crate) resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="action: unbind schema", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UnbindSchemaArgs {
    #[arg(short, long, required = true)]
    pub(crate) schema_name: String,
    #[arg(short, long, required = true)]
    pub(crate) resource_name: String,
}

pub fn process_slow_sub_args(args: SlowSubscribeArgs) -> MqttActionType {
    match args.action {
        SlowSubscribeActionType::Enable(arg) => {
            MqttActionType::SetClusterConfig(SetClusterConfigRequest {
                feature_name: FeatureType::SlowSubscribe.to_string(),
                is_enable: arg.is_enable.unwrap_or(false),
            })
        }
        SlowSubscribeActionType::Set(arg) => {
            MqttActionType::SetSlowSubscribeConfig(SetSlowSubscribeConfigRequest {
                max_store_num: arg.max_store_num,
                delay_type: arg.delay_type.parse().unwrap_or_default(),
            })
        }
        SlowSubscribeActionType::List(arg) => process_list_slow_subscribe_args(arg),
    }
}

fn process_list_slow_subscribe_args(args: ListSlowSubscribeArgs) -> MqttActionType {
    MqttActionType::ListSlowSubscribe(ListSlowSubscribeRequest {
        list: args.list.unwrap_or(100),
        sub_name: args.sub_name.unwrap_or("".to_string()),
        topic: args.topic.unwrap_or("".to_string()),
        client_id: args.client_id.unwrap_or("".to_string()),
        sort: args.sort.unwrap_or(SortType::DESC).to_string(),
    })
}

pub fn process_system_alarm_args(args: SystemAlarmArgs) -> MqttActionType {
    match args.action {
        SystemAlarmActionType::Set(arg) => {
            MqttActionType::SetSystemAlarmConfig(SetSystemAlarmConfigRequest {
                enable: arg.enable,
                os_cpu_high_watermark: arg.cpu_high_watermark,
                os_cpu_low_watermark: arg.cpu_low_watermark,
                os_memory_high_watermark: arg.memory_high_watermark,
                os_cpu_check_interval_ms: arg.os_cpu_check_interval_ms,
            })
        }
        SystemAlarmActionType::List => MqttActionType::ListSystemAlarm(ListSystemAlarmRequest {}),
    }
}

pub fn process_session_args(args: SessionArgs) -> MqttActionType {
    match args.action {
        SessionActionType::List => MqttActionType::ListSession,
    }
}

pub fn process_subscribes_args(args: SubscribesArgs) -> MqttActionType {
    match args.action {
        SubscribesActionType::List => {
            MqttActionType::ListSubscribe(ListSubscribeRequest::default())
        }
        SubscribesActionType::Detail(args) => {
            MqttActionType::DetailSubscribe(SubscribeDetailRequest {
                client_id: args.client_id,
                path: args.path,
            })
        }
    }
}

pub fn process_config_args(args: ClusterConfigArgs) -> MqttActionType {
    match args.action {
        ClusterConfigActionType::Get => MqttActionType::GetClusterConfig,
    }
}

pub fn process_user_args(args: UserArgs) -> MqttActionType {
    match args.action {
        UserActionType::List => MqttActionType::ListUser,
        UserActionType::Create(arg) => MqttActionType::CreateUser(CreateUserRequest {
            username: arg.username,
            password: arg.password,
            is_superuser: arg.is_superuser,
        }),
        UserActionType::Delete(arg) => MqttActionType::DeleteUser(DeleteUserRequest {
            username: arg.username,
        }),
    }
}

pub fn process_acl_args(args: AclArgs) -> Result<MqttActionType, Box<dyn std::error::Error>> {
    match args.action {
        AclActionType::List => Ok(MqttActionType::ListAcl),
        AclActionType::Create(arg) => {
            let acl = MqttAcl {
                resource_type: arg.resource_type,
                resource_name: arg.resource_name,
                topic: arg.topic,
                ip: arg.ip,
                action: arg.action,
                permission: arg.permission,
            };
            Ok(MqttActionType::CreateAcl(CreateAclRequest {
                cluster_name: arg.cluster_name,
                acl: acl.encode()?,
            }))
        }
        AclActionType::Delete(arg) => {
            let acl = MqttAcl {
                resource_type: arg.resource_type,
                resource_name: arg.resource_name,
                topic: arg.topic,
                ip: arg.ip,
                action: arg.action,
                permission: arg.permission,
            };
            Ok(MqttActionType::DeleteAcl(DeleteAclRequest {
                cluster_name: arg.cluster_name,
                acl: acl.encode()?,
            }))
        }
    }
}

pub fn process_blacklist_args(
    args: BlacklistArgs,
) -> Result<MqttActionType, Box<dyn std::error::Error>> {
    match args.action {
        BlackListActionType::List => Ok(MqttActionType::ListBlacklist),
        BlackListActionType::Create(arg) => {
            let mqtt_acl_black_list = MqttAclBlackList {
                blacklist_type: arg.blacklist_type,
                resource_name: arg.resource_name,
                end_time: arg.end_time,
                desc: arg.desc,
            };
            Ok(MqttActionType::CreateBlacklist(CreateBlacklistRequest {
                cluster_name: arg.cluster_name,
                blacklist: mqtt_acl_black_list.encode()?,
            }))
        }
        BlackListActionType::Delete(arg) => {
            Ok(MqttActionType::DeleteBlacklist(DeleteBlacklistRequest {
                cluster_name: arg.cluster_name,
                blacklist_type: arg.blacklist_type.to_string(),
                resource_name: arg.resource_name,
            }))
        }
    }
}

pub fn process_connection_args(args: ConnectionArgs) -> MqttActionType {
    match args.action {
        ConnectionActionType::List => MqttActionType::ListConnection,
    }
}

pub fn process_connector_args(args: ConnectorArgs) -> MqttActionType {
    match args.action {
        ConnectorActionType::List(arg) => MqttActionType::ListConnector(ListConnectorRequest {
            connector_name: arg.connector_name,
        }),
        ConnectorActionType::Create(arg) => {
            MqttActionType::CreateConnector(CreateConnectorRequest {
                connector_name: arg.connector_name,
                connector_type: arg.connector_type.parse().unwrap(),
                config: arg.config,
                topic_id: arg.topic_id,
            })
        }
        ConnectorActionType::Delete(arg) => {
            MqttActionType::DeleteConnector(DeleteConnectorRequest {
                connector_name: arg.connector_name,
            })
        }
        ConnectorActionType::Update(arg) => {
            MqttActionType::UpdateConnector(UpdateConnectorRequest {
                connector: Vec::from(arg.connector),
            })
        }
    }
}

pub fn process_topic_rewrite_args(args: TopicRewriteArgs) -> MqttActionType {
    match args.action {
        TopicRewriteActionType::Create(arg) => {
            MqttActionType::CreateTopicRewriteRule(CreateTopicRewriteRuleRequest {
                action: arg.action,
                source_topic: arg.source_topic,
                dest_topic: arg.dest_topic,
                regex: arg.regex,
            })
        }
        TopicRewriteActionType::Delete(arg) => {
            MqttActionType::DeleteTopicRewriteRule(DeleteTopicRewriteRuleRequest {
                action: arg.action,
                source_topic: arg.source_topic,
            })
        }
    }
}

pub fn process_schema_args(args: SchemaArgs) -> MqttActionType {
    match args.action {
        SchemaActionType::Create(arg) => MqttActionType::CreateSchema(CreateSchemaRequest {
            schema_name: arg.schema_name,
            schema_type: arg.schema_type,
            schema: arg.schema,
            desc: arg.desc,
        }),
        SchemaActionType::List(arg) => MqttActionType::ListSchema(ListSchemaRequest {
            schema_name: arg.schema_name,
        }),
        SchemaActionType::Update(arg) => MqttActionType::UpdateSchema(UpdateSchemaRequest {
            schema_name: arg.schema_name,
            schema_type: arg.schema_type,
            schema: arg.schema,
            desc: arg.desc,
        }),
        SchemaActionType::Delete(arg) => MqttActionType::DeleteSchema(DeleteSchemaRequest {
            schema_name: arg.schema_name,
        }),
        SchemaActionType::ListBind(arg) => MqttActionType::ListBindSchema(ListBindSchemaRequest {
            schema_name: arg.schema_name,
            resource_name: arg.resource_name,
        }),
        SchemaActionType::Bind(arg) => MqttActionType::BindSchema(BindSchemaRequest {
            schema_name: arg.schema_name,
            resource_name: arg.resource_name,
        }),
        SchemaActionType::Unbind(arg) => MqttActionType::UnbindSchema(UnbindSchemaRequest {
            schema_name: arg.schema_name,
            resource_name: arg.resource_name,
        }),
    }
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "related operations of mqtt auto subscribe, such as listing, setting, and deleting", long_about = None
)]
#[command(next_line_help = true)]
pub(crate) struct AutoSubscribeRuleCommand {
    #[command(subcommand)]
    pub action: AutoSubscribeRuleActionType,
}

#[derive(Debug, clap::Subcommand)]
pub enum AutoSubscribeRuleActionType {
    #[command(author = "RobustMQ", about = "action: auto subscribe rule list", long_about = None)]
    List,
    #[command(author = "RobustMQ", about = "action: delete auto subscribe rule", long_about = None)]
    Delete(DeleteAutoSubscribeRuleArgs),
    #[command(author = "RobustMQ", about = "action: set auto subscribe rule", long_about = None)]
    Set(SetAutoSubscribeRuleArgs),
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct SetAutoSubscribeRuleArgs {
    #[arg(short, long, required = true)]
    pub(crate) topic: String,
    #[arg(short, long, default_value_t = 0)]
    pub(crate) qos: u8,
    #[arg(short, long, default_value_t = false)]
    pub(crate) no_local: bool,
    #[arg(short = 'r', long, default_value_t = false)]
    pub(crate) retain_as_published: bool,
    #[arg(short = 'R', long, default_value_t = 0)]
    pub(crate) retained_handling: u8,
}

#[derive(clap::Args, Debug)]
#[command(next_line_help = true)]
pub(crate) struct DeleteAutoSubscribeRuleArgs {
    #[arg(short, long, required = true)]
    pub(crate) topic: String,
}

pub fn process_auto_subscribe_args(args: AutoSubscribeRuleCommand) -> MqttActionType {
    match args.action {
        AutoSubscribeRuleActionType::List => {
            MqttActionType::ListAutoSubscribeRule(ListAutoSubscribeRuleRequest::default())
        }
        AutoSubscribeRuleActionType::Set(arg) => {
            MqttActionType::SetAutoSubscribeRule(SetAutoSubscribeRuleRequest {
                topic: arg.topic,
                qos: arg.qos as u32,
                no_local: arg.no_local,
                retain_as_published: arg.retain_as_published,
                retained_handling: arg.retained_handling as u32,
            })
        }
        AutoSubscribeRuleActionType::Delete(arg) => {
            MqttActionType::DeleteAutoSubscribeRule(DeleteAutoSubscribeRuleRequest {
                topic: arg.topic,
            })
        }
    }
}
