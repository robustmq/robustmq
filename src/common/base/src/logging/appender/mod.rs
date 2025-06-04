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

use std::{collections::HashMap, io};

use serde::Deserialize;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{Layer, Registry};

use crate::error::log_config::LogConfigError;

mod console;
mod rolling_file;

pub(super) use console::*;
pub(super) use rolling_file::*;

// TODO: implement size based rotation

trait AppenderConfig {
    fn create_appender(&self) -> Result<impl io::Write + Send + 'static, LogConfigError>;
}

/// Supported configurations for log appenders.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind")]
enum Appender {
    Console(ConsoleAppenderConfig),
    RollingFile(RollingFileAppenderConfig),
}

impl Appender {
    fn create_non_blocking_writer(self) -> Result<(NonBlocking, WorkerGuard), LogConfigError> {
        match self {
            Appender::Console(console_appender_config) => {
                let writer = console_appender_config.create_appender()?;
                let (non_blocking, guard) = tracing_appender::non_blocking(writer);
                Ok((non_blocking, guard))
            }
            Appender::RollingFile(rolling_file_appender_config) => {
                let writer = rolling_file_appender_config.create_appender()?;
                let (non_blocking, guard) = tracing_appender::non_blocking(writer);
                Ok((non_blocking, guard))
            },
        }
    }
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
pub(super) struct Config {
    #[serde(flatten)]
    appender: Appender,
    level: Level,
}

type BoxedLayer<S = Registry> = Box<dyn Layer<S> + Send + Sync + 'static>;

fn fmt_layer<S>(with_ansi: bool) -> tracing_subscriber::fmt::Layer<S> {
    tracing_subscriber::fmt::layer().with_ansi(with_ansi)
}

impl Config {
    pub(super) fn create_layer(self) -> Result<(BoxedLayer, WorkerGuard), LogConfigError> {
        let level: tracing::Level = self.level.into();
        let (writer, guard) = self.appender.create_non_blocking_writer()?;
        let layer = fmt_layer(true)
            .with_writer(writer)
            .with_filter(tracing_subscriber::filter::LevelFilter::from(level))
            .boxed();
        Ok((layer, guard))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub(super) struct Configs {
    pub(super) appenders: HashMap<String, Config>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEBUG_LEVEL_TOML: &str = r#"
        level = "Debug"
        "#;
    
    const CONSOLE_KIND_TOML: &str = r#"
        kind = "Console"
        "#;
    const CONSOLE_CONFIG_TOML: &str = r#""#;
    
    const ROLLING_FILE_KIND_TOML: &str = r#"
        kind = "RollingFile"
        "#;
    const ROLLING_FILE_CONFIG_TOML: &str = r#"
        rotation = "Daily"
        directory = "/var/logs"
        prefix = "app"
        "#;

    #[test]
    fn test_deserialize_console_appender_toml() {
        let config_toml = format!(
            "{level}{kind}{config}",
            level = DEBUG_LEVEL_TOML,
            kind = CONSOLE_KIND_TOML,
            config = CONSOLE_CONFIG_TOML
        );

        let config: super::Config = toml::from_str(&config_toml).unwrap();

        assert!(matches!(config.appender, Appender::Console(_)));
        assert_eq!(config.level, Level::Debug);
    }

    #[test]
    fn test_deserialize_rolling_file_appender_toml() {
        let config_toml = format!(
            "{level}{kind}{config}",
            level = DEBUG_LEVEL_TOML,
            kind = ROLLING_FILE_KIND_TOML,
            config = ROLLING_FILE_CONFIG_TOML
        );

        let config: super::Config = toml::from_str(&config_toml).unwrap();

        assert_eq!(config.level, Level::Debug);

        let expected: RollingFileAppenderConfig = toml::from_str(ROLLING_FILE_CONFIG_TOML).unwrap();
        if let Appender::RollingFile(found) = config.appender {
            assert_eq!(found, expected);
        } else {
            panic!("Expected RollingFile appender");
        }
    }
}
