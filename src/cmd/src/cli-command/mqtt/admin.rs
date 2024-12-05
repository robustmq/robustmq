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
use common_base::enum_type::common_enum::SortType;

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

#[allow(dead_code)]
impl SlowSubArgs {
    pub fn get_is_enable(&self) -> Option<bool> {
        self.is_enable
    }

    pub fn get_list(&self) -> Option<u64> {
        self.list
    }

    pub fn get_sort(&self) -> Option<SortType> {
        self.sort
    }

    pub fn get_topic(&self) -> Option<String> {
        self.topic.clone()
    }

    pub fn get_sub_name(&self) -> Option<String> {
        self.sub_name.clone()
    }

    pub fn get_client_id(&self) -> Option<String> {
        self.client_id.clone()
    }
}
