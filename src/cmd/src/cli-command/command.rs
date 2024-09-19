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
use cli_command::{CliCommand, CliCommandAParam};

#[derive(Parser, Debug)]
#[command(author="robustmq-command", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]

struct CliCommandArgsParams {
    service: String,
    action: String,
}

fn main() {
    let args = CliCommandArgsParams::parse();
    let cli = CliCommand::new();
    let params = CliCommandAParam {
        service: args.service,
        action: args.action,
    };
    cli.start(params);
}
