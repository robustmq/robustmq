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

use std::collections::HashMap;

use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Layer, Registry};

use crate::error::log_config::LogConfigError;

mod console;
mod rolling_file;

pub(super) use console::*;
pub(super) use rolling_file::*;

// TODO: implement size based rotation


#[derive(Debug, Clone, Deserialize, PartialEq)]
enum Kind {
    Console,
    RollingFile {
        config: RollingFileAppenderConfig
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for tracing::Level {
    fn from(value: Level) -> Self {
        match value {
            Level::Error => tracing::Level::ERROR,
            Level::Warn => tracing::Level::WARN,
            Level::Info => tracing::Level::INFO,
            Level::Debug => tracing::Level::DEBUG,
            Level::Trace => tracing::Level::TRACE,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct AppenderConfig {
    kind: Kind,
    level: Level,

    // // Optional fields for RollingFile appender
    // // TODO: whether we want to validate these fields for console appender?
    // pub(super) rotation: Option<Rotation>,
    // pub(super) directory: Option<String>,
    // pub(super) prefix: Option<String>,
    // pub(super) suffix: Option<String>,
    // pub(super) max_log_files: Option<u32>,
}

type BoxedLayer<S = Registry> = Box<dyn Layer<S> + Send + Sync + 'static>;

fn fmt_layer<S>(with_ansi: bool) -> tracing_subscriber::fmt::Layer<S> {
    tracing_subscriber::fmt::layer().with_ansi(with_ansi)
}

impl AppenderConfig {
    pub(super) fn try_into_layer_and_guard(
        self,
    ) -> Result<(BoxedLayer, Option<WorkerGuard>), LogConfigError> {
        match self.kind {
            Kind::Console => {
                let level: tracing::Level = self.level.into();
                // TODO: formatting pretty or compact?
                let layer = fmt_layer(true)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from(level))
                    .boxed();
                Ok((layer, None))
            }
            Kind::RollingFile { config } => {
                let rolling = config.create_appender()?;
                let level: tracing::Level = self.level.into();

                // TODO: do we want to use non-blocking writer here? If panic occurs, some events may be lost.
                let (non_blocking, guard) = tracing_appender::non_blocking(rolling);

                let layer = fmt_layer(false)
                    .with_writer(non_blocking)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from(level))
                    .boxed();
                Ok((layer, Some(guard)))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Config {
    pub(super) appenders: HashMap<String, AppenderConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_console_appender_toml() {
        let config_toml = r#"
        [appenders.stdout]
        kind = "Console"
        level = "Info"
        "#;

        let config: super::Config = toml::from_str(config_toml).unwrap();
        assert_eq!(config.appenders.len(), 1);

        let stdout_appender = config.appenders.get("stdout").unwrap();
        assert_eq!(stdout_appender.kind, Kind::Console);
        assert_eq!(stdout_appender.level, Level::Info);
    }

    // #[test]
    // fn test_single_rolling_file_appender_toml() {
    //     let inline_config_toml = r#"
    //     [appenders]
    //     file = { kind = "RollingFile", level = "Debug", rotation = "Daily", directory = "/var/logs", prefix = "app" }
    //     "#;

    //     let config: super::Config = toml::from_str(inline_config_toml).unwrap();
    //     assert_eq!(config.appenders.len(), 1);
    //     let file_appender = config.appenders.get("file").unwrap();
    //     assert_eq!(file_appender.kind, appender::AppenderKind::RollingFile);
    //     assert_eq!(file_appender.level, appender::Level::Debug);
    //     assert_eq!(file_appender.rotation, Some(appender::Rotation::Daily));
    //     assert_eq!(file_appender.directory, Some("/var/logs".to_string()));
    //     assert_eq!(file_appender.prefix, Some("app".to_string()));

    //     let multiline_config_toml = r#"
    //     [appenders.file]
    //     kind = "RollingFile"
    //     level = "Debug"
    //     rotation = "Daily"
    //     directory = "/var/logs"
    //     prefix = "app"
    //     "#;

    //     let config: super::Config = toml::from_str(multiline_config_toml).unwrap();
    //     assert_eq!(config.appenders.len(), 1);
    //     let file_appender = config.appenders.get("file").unwrap();
    //     assert_eq!(file_appender.kind, appender::AppenderKind::RollingFile);
    //     assert_eq!(file_appender.level, appender::Level::Debug);
    //     assert_eq!(file_appender.rotation, Some(appender::Rotation::Daily));
    //     assert_eq!(file_appender.directory, Some("/var/logs".to_string()));
    //     assert_eq!(file_appender.prefix, Some("app".to_string()));
    // }

    // #[test]
    // fn test_multiple_appenders_toml() {
    //     let inline_config_toml = r#"
    //     [appenders]
    //     stdout = { kind = "Console", level = "Info" }
    //     file = { kind = "RollingFile", level = "Debug", rotation = "Daily", directory = "/var/logs", prefix = "app" }
    //     "#;

    //     let config: super::Config = toml::from_str(inline_config_toml).unwrap();
    //     assert_eq!(config.appenders.len(), 2);
    //     let stdout_appender = config.appenders.get("stdout").unwrap();
    //     assert_eq!(stdout_appender.kind, appender::AppenderKind::Console);
    //     assert_eq!(stdout_appender.level, appender::Level::Info);
    //     assert_eq!(stdout_appender.rotation, None);
    //     assert_eq!(stdout_appender.directory, None);
    //     assert_eq!(stdout_appender.prefix, None);
    //     let file_appender = config.appenders.get("file").unwrap();
    //     assert_eq!(file_appender.kind, appender::AppenderKind::RollingFile);
    //     assert_eq!(file_appender.level, appender::Level::Debug);
    //     assert_eq!(file_appender.rotation, Some(appender::Rotation::Daily));
    //     assert_eq!(file_appender.directory, Some("/var/logs".to_string()));
    //     assert_eq!(file_appender.prefix, Some("app".to_string()));

    //     let mixed_config_toml = r#"
    //     [appenders]
    //     stdout = { kind = "Console", level = "Info" }

    //     [appenders.file]
    //     kind = "RollingFile"
    //     level = "Debug"
    //     rotation = "Daily"
    //     directory = "/var/logs"
    //     prefix = "app"
    //     "#;

    //     let config: super::Config = toml::from_str(mixed_config_toml).unwrap();
    //     assert_eq!(config.appenders.len(), 2);
    //     let stdout_appender = config.appenders.get("stdout").unwrap();
    //     assert_eq!(stdout_appender.kind, appender::AppenderKind::Console);
    //     assert_eq!(stdout_appender.level, appender::Level::Info);
    //     assert_eq!(stdout_appender.rotation, None);
    //     assert_eq!(stdout_appender.directory, None);
    //     assert_eq!(stdout_appender.prefix, None);
    //     let file_appender = config.appenders.get("file").unwrap();
    //     assert_eq!(file_appender.kind, appender::AppenderKind::RollingFile);
    //     assert_eq!(file_appender.level, appender::Level::Debug);
    //     assert_eq!(file_appender.rotation, Some(appender::Rotation::Daily));
    //     assert_eq!(file_appender.directory, Some("/var/logs".to_string()));
    //     assert_eq!(file_appender.prefix, Some("app".to_string()));
    // }
}
