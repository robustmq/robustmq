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

use clap::command;
use clap::Parser;
use common_base::config::placement_center::init_placement_center_conf_by_path;
use common_base::config::DEFAULT_PLACEMENT_CENTER_CONFIG;
use common_base::logs::init_placement_center_log;
use placement_center::PlacementCenter;
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author="robustmq", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]

struct ArgsParams {
    /// MetaService Indicates the path of the configuration file
    #[arg(short, long, default_value_t=String::from(DEFAULT_PLACEMENT_CENTER_CONFIG))]
    conf: String,
}
#[tokio::main]
async fn main() {
    let args = ArgsParams::parse();
    init_placement_center_conf_by_path(&args.conf);
    init_placement_center_log();
    let (stop_send, _) = broadcast::channel(2);
    let mut pc = PlacementCenter::new();
    pc.start(stop_send).await;
}
