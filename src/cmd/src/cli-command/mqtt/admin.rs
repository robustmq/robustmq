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
use clap::Parser;
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
    #[arg(long)]
    #[arg(value_parser = RangedU64ValueParser::<u64>::new())]
    #[arg(default_missing_value = "100")]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(require_equals = true)]
    list: Option<u64>,
    #[arg(long)]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(value_parser = EnumValueParser::<SortType>::new())]
    #[arg(default_missing_value = "DESC", ignore_case = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(help = "Sort the results")]
    sort: Option<SortType>,
    #[arg(long)]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    topic: Option<String>,
    #[arg(long)]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    sub_name: Option<String>,
    #[arg(long)]
    #[arg(required = false, requires = "list", require_equals = true)]
    #[arg(action = ArgAction::Set, num_args = 0..=1)]
    #[arg(value_parser = NonEmptyStringValueParser::new())]
    client_id: Option<String>,
}

pub fn process_slow_sub_args(args: SlowSubArgs) -> MqttActionType {
    if args.is_enable.is_none() {
        MqttActionType::ListSlowSubscribe(ListSlowSubscribeRequest {
            list: args.list.unwrap_or(100),
            sub_name: args.sub_name.unwrap_or("".to_string()),
            topic: args.topic.unwrap_or("".to_string()),
            client_id: args.client_id.unwrap_or("".to_string()),
        })
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
    async fn test_process_slow_sub_args_function_filed_is_enable_none() {
        let args = SlowSubArgs {
            is_enable: None,
            list: None,
            sort: None,
            topic: None,
            sub_name: None,
            client_id: None,
        };

        let action_type = process_slow_sub_args(args);
        assert_eq!(
            MqttActionType::ListSlowSubscribe(ListSlowSubscribeRequest {
                list: 100,
                sub_name: "".to_string(),
                topic: "".to_string(),
                client_id: "".to_string()
            }),
            action_type
        )
    }

    #[tokio::test]
    async fn test_process_slow_sub_args_function_filed_is_enable_none_1() {
        let args = SlowSubArgs {
            is_enable: None,
            list: None,
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
                client_id: "client_id".to_string()
            }),
            action_type
        )
    }
}
