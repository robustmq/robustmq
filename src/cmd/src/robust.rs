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

use broker::broker::Broker;
use clap::command;
use clap::Parser;
use common_config::{
    meta::MetaConfig, server::RobustConfig, DEFAULT_META_CONFIG, DEFAULT_SERVER_CONFIG,
};
use common_log::log::error;
use common_log::log::info;
use common_log::log;
use common_metrics::server::ServerMetrics;
use lazy_static::lazy_static;
use tokio::signal;

#[derive(Parser, Debug)]
#[command(author="robustmq", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]

struct ArgsParams {
    /// broker server configuration file path
    #[arg(short, long, default_value_t=String::from(DEFAULT_SERVER_CONFIG))]
    server_conf: String,

    /// MetaService Indicates the path of the configuration file
    #[arg(short, long, default_value_t=String::from(DEFAULT_META_CONFIG))]
    meta_conf: String,
}

lazy_static! {
    static ref SERVER_METRICS: ServerMetrics = ServerMetrics::new();
}

fn main() {
    let args = ArgsParams::parse();
    log::new();

    let server_conf: RobustConfig = common_config::parse_server(&args.server_conf);
    let meta_conf: MetaConfig = common_config::parse_meta(&args.meta_conf);
    
    let app: Broker = Broker::new(10, 10,10,10);
    app.start().await;
    // tokio::select! {
    //     result = app.start() => {
    //         if let Err(err) = result {
    //             error(&format!("Fatal error occurs!,err:{:?}",err));
    //         }
    //     }
    //     _ = signal::ctrl_c() => {
    //         info("Listen for stop signal Ctrl+C...");
    //         if let Err(err) = app.stop().await {
    //             error(&format!("Fatal error occurs!,err:{:?}",err));
    //         }
    //         info("Goodbye!");
    //     }
    // }
}
