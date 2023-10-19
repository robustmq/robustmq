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
use tokio::{runtime::Runtime, signal};
use lazy_static::lazy_static;

mod admin;
mod config;
mod log;
mod metrics;

struct ArgsParams {
    config_path: String,
}

lazy_static! {
    static ref SERVER_METRICS:ServerMetrics = ServerMetrics::new();
}

fn main() {
    log::new();

    let args = parse_args();
    let conf: config::RobustConfig = config::new(&args.config_path);
    SERVER_METRICS.init();

    let admin_server = admin::AdminServer::new(&conf);
    let admin_runtime = admin_server.start();

    SERVER_METRICS.set_server_status_running();
    log::server_info("RobustMQ Server was successfully started");

    shutdown_hook(admin_runtime);
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

async fn _signal_hook() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        c1 = ctrl_c => {println!("{:?}",c1)},
        c2 = terminate => {println!("{:?}",c2)},
    }
}

fn shutdown_hook(admin_runtime: Runtime) {
    loop {
        thread::sleep(Duration::from_secs(1000));
        break;
    }
    admin_runtime.shutdown_timeout(Duration::from_secs(1000));
}
