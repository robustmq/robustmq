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

use clap::{command, Parser};
use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
use common_base::config::DEFAULT_MQTT_SERVER_CONFIG;
use common_base::logging::init_broker_mqtt_log;
use mqtt_broker::start_mqtt_broker_server;
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author="robustmq", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]
struct ArgsParams {
    /// broker server configuration file path
    #[arg(short, long, default_value_t=String::from(DEFAULT_MQTT_SERVER_CONFIG))]
    conf: String,
}

fn main() {
    let args = ArgsParams::parse();
    init_broker_mqtt_conf_by_path(&args.conf);
    // Need to keep the guard alive until the application terminates
    let _appender_guards = init_broker_mqtt_log().unwrap();
    let (stop_send, _) = broadcast::channel(2);
    start_mqtt_broker_server(stop_send);
}
