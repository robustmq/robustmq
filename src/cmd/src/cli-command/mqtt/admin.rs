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

use core::option::Option::Some;

use clap::builder::{
    ArgAction, BoolishValueParser, EnumValueParser, NonEmptyStringValueParser, RangedU64ValueParser,
};
use clap::{arg, Parser};
use cli_command::mqtt::MqttActionType;
use common_base::enum_type::sort_type::SortType;
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateUserRequest, DeleteAutoSubscribeRuleRequest, DeleteUserRequest,
    ListAutoSubscribeRuleRequest, SetAutoSubscribeRuleRequest,
};
use protocol::broker_mqtt::broker_mqtt_admin::{
    EnableSlowSubscribeRequest, ListSlowSubscribeRequest,
};

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="related operations of mqtt users, such as listing, creating, and deleting ", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UserArgs {
    #[command(subcommand)]
    pub action: Option<UserActionType>,
}

#[derive(Debug, clap::Subcommand)]
pub enum UserActionType {
    #[command(author="RobustMQ", about="action: list users", long_about = None)]
    List,
    #[command(author="RobustMQ", about="action: create user", long_about = None)]
    Create(CreateUserArgs),
    #[command(author="RobustMQ", about="action: delete user", long_about = None)]
    Delete(DeleteUserArgs),
}

// security: user feat
#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: create user", long_about = None)]
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
#[command(author="RobustMQ", about="action: delete user", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteUserArgs {
    #[arg(short, long, required = true)]
    pub(crate) username: String,
}

// flapping detect feat
#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct FlappingDetectArgs {
    #[arg(long = "enable")]
    #[arg(value_parser =  BoolishValueParser::new())]
    #[arg(default_missing_value = "false")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(required = true, require_equals = true, exclusive = true)]
    #[arg(help = "Enable or disable the feature")]
    pub(crate) is_enable: Option<bool>,
    #[arg(long = "window-time")]
    #[arg(value_parser = RangedU64ValueParser::<u32>::new())]
    #[arg(default_missing_value = "1")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(require_equals = true)]
    #[arg(help = "unit is minutes")]
    pub(crate) window_time: Option<u32>,
    #[arg(long = "max-client-connections")]
    #[arg(value_parser = RangedU64ValueParser::<u32>::new())]
    #[arg(default_missing_value = "15")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    pub(crate) max_client_connections: Option<u32>,
    #[arg(long = "ban-time")]
    #[arg(value_parser = RangedU64ValueParser::<u32>::new())]
    #[arg(default_missing_value = "5")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(help = "unit is minutes")]
    pub(crate) ban_time: Option<u32>,
}

// observability: slow-sub feat
#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct SlowSubArgs {
    #[arg(long = "enable")]
    #[arg(value_parser =  BoolishValueParser::new())]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(required = false, require_equals = true, exclusive = true)]
    #[arg(conflicts_with_all = ["list", "sort", "topic", "sub_name", "client_id"])]
    #[arg(help = "Enable or disable the feature")]
    is_enable: Option<bool>,
    #[arg(long = "list")]
    #[arg(value_parser = RangedU64ValueParser::<u64>::new())]
    #[arg(default_missing_value = "100")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(require_equals = true)]
    list: Option<u64>,
    #[arg(long = "sort")]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(value_parser = EnumValueParser::<SortType>::new())]
    #[arg(default_missing_value = "DESC", ignore_case = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(help = "Sort the results")]
    sort: Option<SortType>,
    #[arg(long = "topic")]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    topic: Option<String>,
    #[arg(long = "sub-name")]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    sub_name: Option<String>,
    #[arg(long = "client-id")]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    client_id: Option<String>,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct ListConnectorArgs {
    pub(crate) connector_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct CreateConnectorArgs {
    pub(crate) connector_name: String,
    pub(crate) connector_type: i32,
    pub(crate) config: String,
    pub(crate) topic_id: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UpdateConnectorArgs {
    pub(crate) connector: Vec<u8>,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteConnectorArgs {
    pub(crate) connector_name: String,
}

// schema
#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct ListSchemaArgs {
    pub(crate) schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct CreateSchemaArgs {
    pub(crate) schema_name: String,
    pub(crate) schema_type: String,
    pub(crate) schema: String,
    pub(crate) desc: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UpdateSchemaArgs {
    pub(crate) schema_name: String,
    pub(crate) schema_type: String,
    pub(crate) schema: String,
    pub(crate) desc: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteSchemaArgs {
    pub(crate) schema_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct ListBindSchemaArgs {
    pub(crate) schema_name: String,
    pub(crate) resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct BindSchemaArgs {
    pub(crate) schema_name: String,
    pub(crate) resource_name: String,
}

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct UnbindSchemaArgs {
    pub(crate) schema_name: String,
    pub(crate) resource_name: String,
}

pub fn process_slow_sub_args(args: SlowSubArgs) -> MqttActionType {
    if args.is_enable.is_none() {
        if args.list.is_none() {
            panic!("Please provide at least one argument(--is-enable | --list). Use --help for more information.")
        } else {
            MqttActionType::ListSlowSubscribe(ListSlowSubscribeRequest {
                list: args.list.unwrap_or(100),
                sub_name: args.sub_name.unwrap_or("".to_string()),
                topic: args.topic.unwrap_or("".to_string()),
                client_id: args.client_id.unwrap_or("".to_string()),
                sort: args.sort.unwrap_or(SortType::DESC).to_string(),
            })
        }
    } else {
        MqttActionType::EnableSlowSubscribe(EnableSlowSubscribeRequest {
            is_enable: args.is_enable.unwrap(),
        })
    }
}

pub fn process_user_args(args: UserArgs) -> MqttActionType {
    match args.action {
        Some(user_action) => match user_action {
            UserActionType::List => MqttActionType::ListUser,
            UserActionType::Create(arg) => MqttActionType::CreateUser(CreateUserRequest {
                username: arg.username,
                password: arg.password,
                is_superuser: arg.is_superuser,
            }),
            UserActionType::Delete(arg) => MqttActionType::DeleteUser(DeleteUserRequest {
                username: arg.username,
            }),
        },
        None => unreachable!(),
    }
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="related operations of mqtt auto subscribe, such as listing, setting, and deleting ", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct MqttAutoSubscribeRuleCommand {
    #[command(subcommand)]
    pub action: Option<MqttAutoSubscribeRuleActionType>,
}

#[derive(Debug, clap::Subcommand)]
pub enum MqttAutoSubscribeRuleActionType {
    #[command(author="RobustMQ", about="action: user list", long_about = None)]
    List,
    Delete(DeleteAutoSubscribeRuleArgs),
    Set(SetAutoSubscribeRuleArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: set auto subscribe rule", long_about = None)]
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
#[command(author="RobustMQ", about="action: delete auto subscribe rule", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct DeleteAutoSubscribeRuleArgs {
    #[arg(short, long, required = true)]
    pub(crate) topic: String,
}

pub fn process_auto_subscribe_args(args: MqttAutoSubscribeRuleCommand) -> MqttActionType {
    match args.action {
        Some(auto_subscribe_action) => match auto_subscribe_action {
            MqttAutoSubscribeRuleActionType::List => {
                MqttActionType::ListAutoSubscribeRule(ListAutoSubscribeRuleRequest::default())
            }
            MqttAutoSubscribeRuleActionType::Set(arg) => {
                MqttActionType::SetAutoSubscribeRule(SetAutoSubscribeRuleRequest {
                    topic: arg.topic,
                    qos: arg.qos as u32,
                    no_local: arg.no_local,
                    retain_as_published: arg.retain_as_published,
                    retained_handling: arg.retained_handling as u32,
                })
            }
            MqttAutoSubscribeRuleActionType::Delete(arg) => {
                MqttActionType::DeleteAutoSubscribeRule(DeleteAutoSubscribeRuleRequest {
                    topic: arg.topic,
                })
            }
        },
        None => unreachable!(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_process_slow_sub_args_function_field_is_enable_not_none() {
        let args = SlowSubArgs {
            is_enable: Some(true),
            list: None,
            sort: None,
            topic: None,
            sub_name: None,
            client_id: None,
        };

        let action_type = process_slow_sub_args(args);
        assert_eq!(
            MqttActionType::EnableSlowSubscribe(EnableSlowSubscribeRequest { is_enable: true }),
            action_type
        )
    }

    #[tokio::test]
    #[should_panic(
        expected = "Please provide at least one argument(--is-enable | --list). Use --help for more information."
    )]
    async fn test_process_slow_sub_args_function_filed_is_enable_none() {
        let args = SlowSubArgs {
            is_enable: None,
            list: None,
            sort: None,
            topic: None,
            sub_name: None,
            client_id: None,
        };

        process_slow_sub_args(args);
    }

    #[tokio::test]
    #[should_panic(
        expected = "Please provide at least one argument(--is-enable | --list). Use --help for more information."
    )]
    async fn test_process_slow_sub_args_function_filed_is_enable_none_1() {
        let args = SlowSubArgs {
            is_enable: None,
            list: None,
            sort: Some(SortType::ASC),
            topic: Some("topic_name".to_string()),
            sub_name: Some("sub_name".to_string()),
            client_id: Some("client_id".to_string()),
        };

        process_slow_sub_args(args);
    }

    #[tokio::test]
    async fn test_process_slow_sub_args_function_filed_is_enable_none_2() {
        let args = SlowSubArgs {
            is_enable: None,
            list: Some(100),
            sort: Some(SortType::ASC),
            topic: Some("topic_name".to_string()),
            sub_name: Some("sub_name".to_string()),
            client_id: Some("client_id".to_string()),
        };

        let action_type = process_slow_sub_args(args);
        assert_eq!(
            MqttActionType::ListSlowSubscribe(ListSlowSubscribeRequest {
                list: 100,
                sub_name: "sub_name".to_string(),
                topic: "topic_name".to_string(),
                client_id: "client_id".to_string(),
                sort: "asc".to_string(),
            }),
            action_type
        )
    }
}
