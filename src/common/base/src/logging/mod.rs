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

use std::path::Path;

use crate::error::log_config::LogConfigError;
use crate::tools::{file_exists, read_file, try_create_fold};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod console;
mod filter;
mod fmt;
mod rolling_file;
mod tokio_console;

/// Initializes the tracing subscriber with the specified log configuration file
/// and log path.
///
/// Returns a vector of `WorkerGuard` instances for the non-blocking file
/// appender(s) if there is/are any. The guards manage the background thread
/// that writes log events to the file and must be kept alive until the
/// application terminates.
pub fn init_tracing_subscriber(
    log_config_file: impl AsRef<Path>,
    log_path: impl AsRef<Path>,
) -> Result<Vec<WorkerGuard>, LogConfigError> {
    let log_config_file = log_config_file.as_ref();
    let log_path = log_path.as_ref();

    if !file_exists(log_config_file) {
        panic!("Logging configuration file {log_config_file:?} does not exist");
    }

    let content = match read_file(log_config_file) {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    match try_create_fold(log_path) {
        Ok(()) => {}
        Err(_) => {
            panic!("Failed to initialize log directory {log_path:?}");
        }
    }
    let config: config::Configs = toml::from_str(&content)?;
    init_tracing_subscriber_with_config(config)
}

fn init_tracing_subscriber_with_config(
    config: config::Configs,
) -> Result<Vec<WorkerGuard>, LogConfigError> {
    let mut layers = Vec::with_capacity(config.appenders.len());
    let mut guards = Vec::with_capacity(config.appenders.len());

    for (_name, conf) in config.appenders {
        let (layer, guard) = conf.create_layer_and_guard()?;

        layers.push(layer);

        if let Some(guard) = guard {
            guards.push(guard);
        }
    }

    let registry = tracing_subscriber::registry().with(layers);
    registry.init();

    Ok(guards)
}
