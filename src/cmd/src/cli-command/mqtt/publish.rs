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

use clap::{arg, command, Parser};
use cli_command::mqtt::MqttActionType;
use cli_command::template::{PublishArgsRequest, SubscribeArgsRequest};

#[derive(Debug, Parser)]
#[command(author="RobustMQ", about="Command line tool for mqtt broker", long_about = None)]
#[command(next_line_help = true)]
pub(crate) struct PubSubArgs {
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

pub(crate) fn process_publish_args(args: PubSubArgs) -> MqttActionType {
    MqttActionType::Publish(PublishArgsRequest {
        username: args.username,
        password: args.password,
        topic: args.topic,
        qos: args.qos,
        retained: args.retained,
    })
}

pub(crate) fn process_subscribe_args(args: PubSubArgs) -> MqttActionType {
    MqttActionType::Subscribe(SubscribeArgsRequest {
        username: args.username,
        password: args.password,
        topic: args.topic,
        qos: args.qos,
    })
}
