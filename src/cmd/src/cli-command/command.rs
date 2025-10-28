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

use clap::Parser;
use cli_command::{
    cluster::command::ClusterCommand,
    handler::{
        handle_cluster, handle_journal, handle_mqtt, handle_status, RobustMQCli, RobustMQCliCommand,
    },
    mqtt::command::MqttBrokerCommand,
};
use common_base::version::logo::banner_print;

#[tokio::main]
async fn main() {
    print!("{:?}", banner_print());
    let args = RobustMQCli::parse();
    match args.command {
        RobustMQCliCommand::Mqtt(args) => handle_mqtt(args, MqttBrokerCommand::new()).await,
        RobustMQCliCommand::Cluster(args) => handle_cluster(args, ClusterCommand::new()).await,
        RobustMQCliCommand::Journal(args) => handle_journal(args).await,
        RobustMQCliCommand::Status(args) => handle_status(args).await,
    }
}
