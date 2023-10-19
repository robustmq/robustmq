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

use metrics::ServerMetrics;
use std::{env, thread, time::Duration};

mod admin;
mod config;
mod log;
mod metrics;

struct ArgsParams {
    config_path: String,
}

fn main() {
    log::new();

    let args = parse_args();
    let conf: config::RobustConfig = config::new(&args.config_path);

    let server_metrics: ServerMetrics = ServerMetrics::new();
    server_metrics.init();

    let admin_server = admin::AdminServer::new(
        conf.addr.clone(),
        conf.admin.port,
        conf.admin.work_thread.unwrap() as usize,
        &server_metrics,
    );
    admin_server.start();

    server_metrics.set_server_status_running();
    log::server_info("RobustMQ Server was successfully started");

    shutdown_hook();
    admin_server.stop();
}

fn parse_args() -> ArgsParams {
    let args: Vec<String> = env::args().collect();
    let mut config_path = config::DEFAULT_SERVER_CONFIG;

    if args.len() > 1 {
        config_path = &args[1];
    }

    return ArgsParams {
        config_path: config_path.to_string(),
    };
}

fn shutdown_hook() {
    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
