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
use cli_command::handler::{
    handle_cluster, handle_engine, handle_mqtt, handle_status, RobustMQCli, RobustMQCliCommand,
};
use common_base::version::logo::banner_print;

#[tokio::main]
async fn main() {
    banner_print();
    let args = RobustMQCli::parse();
    match args.command {
        RobustMQCliCommand::Cluster(args) => handle_cluster(args).await,
        RobustMQCliCommand::Mqtt(args) => handle_mqtt(args).await,
        RobustMQCliCommand::Engine(args) => handle_engine(args).await,
        RobustMQCliCommand::Status(args) => handle_status(args).await,
    }
}
