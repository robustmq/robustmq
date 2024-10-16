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
use cli_command::mqtt::{MqttBrokerCommand, MqttCliCommandParam};

#[derive(Parser, Debug)]
#[command(author="RobustMQ", version="0.0.1", about="Command line tool for mqtt broker", long_about = None)]
#[command(next_line_help = true)]
struct CliCommandArgsParams {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:9981"))]
    server: String,

    #[arg(short, long,default_value_t =String::from("status"))]
    action: String,
}

#[tokio::main]
async fn main() {
    let args = CliCommandArgsParams::parse();
    let cmd = MqttBrokerCommand::new();
    let params = MqttCliCommandParam {
        server: args.server,
        action: args.action,
    };
    cmd.start(params).await;
}
