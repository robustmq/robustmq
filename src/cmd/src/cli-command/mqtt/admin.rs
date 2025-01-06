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
use common_base::enum_type::common_enum::SortType;
use protocol::broker_mqtt::broker_mqtt_admin::{
    EnableSlowSubscribeRequest, ListSlowSubscribeRequest,
};

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

// flapping-detect feat
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
    #[arg(long = "max-disconnect")]
    #[arg(value_parser = RangedU64ValueParser::<u32>::new())]
    #[arg(default_missing_value = "15")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    pub(crate) max_disconnects: Option<u32>,
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
