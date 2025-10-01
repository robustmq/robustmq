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

use broker_server::{
    common::{init_broker_log, print_conf},
    BrokerServer,
};
use clap::{command, Parser};
use common_base::version::logo::banner_info;
use common_config::{broker::init_broker_conf_by_path, DEFAULT_BROKER_SERVER_CONFIG};

#[derive(Parser, Debug)]
#[command(author="RobustMQ", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]
struct ArgsParams {
    /// broker server configuration file path
    #[arg(short, long, default_value_t=String::from(DEFAULT_BROKER_SERVER_CONFIG))]
    conf: String,
}

fn main() {
    let args = ArgsParams::parse();
    init_broker_conf_by_path(&args.conf);
    #[allow(unused_variables)]
    let append = init_broker_log().unwrap();
    banner_info();
    print_conf();
    let server = BrokerServer::new();
    server.start();
}
