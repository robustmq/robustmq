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

use admin;
use config;
use metrics;
use rlog;
use meta;
use std::env;

struct ArgsParams {
    config_path: String,
}

fn main() {
    rlog::new();

    let args = parse_args();
    let conf: config::RobustServerConfig = config::new(&args.config_path);

    let admin_handle = admin::start(conf.addr.clone(), conf.admin.port,conf.admin.work_thread.unwrap() as usize);

    meta::start();

    metrics::SERVER_METRICS.set_server_status_running();
    rlog::server_info("RobustMQ Server was successfully started");

    admin_handle.join().unwrap();
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
