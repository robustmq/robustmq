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

use crate::{
    error::log_config::LogConfigError,
    logging::{
        console::ConsoleAppenderConfig, rolling_file::RollingFileAppenderConfig,
        tokio_console::TokioConsoleAppenderConfig,
    },
};

// TODO: implement size based rotation

pub(super) trait AppenderConfig<S = Registry>
where
    S: tracing::Subscriber,
{
    fn create_layer_and_guard(
        &self,
    ) -> Result<(BoxedLayer<S>, Option<WorkerGuard>), LogConfigError>;
}

/// Supported configurations for log appenders.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub(super) enum Appender {
    Console(ConsoleAppenderConfig),
    RollingFile(RollingFileAppenderConfig),
    TokioConsole(TokioConsoleAppenderConfig),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

impl Appender {
    pub(super) fn create_layer_and_guard<S>(
        self,
    ) -> Result<(BoxedLayer<S>, Option<WorkerGuard>), LogConfigError>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        match self {
            Appender::Console(console_appender_config) => {
                console_appender_config.create_layer_and_guard()
            }
            Appender::RollingFile(rolling_file_appender_config) => {
                rolling_file_appender_config.create_layer_and_guard()
            }
            Appender::TokioConsole(tokio_console_appender_config) => {
                tokio_console_appender_config.create_layer_and_guard()
            }
        }
    }
}

pub(super) type BoxedLayer<S = Registry> = Box<dyn Layer<S> + Send + Sync + 'static>;

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub(super) struct Configs {
    pub(super) appenders: HashMap<String, Appender>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEBUG_LEVEL_TOML: &str = r#"
        level = "Debug"
        "#;

    const CONSOLE_TABLE_NAME: &str = r#"stdout"#;
    const CONSOLE_KIND_TOML: &str = r#"
        kind = "Console"
        "#;
    const CONSOLE_CONFIG_TOML: &str = r#""#;

    const ROLLING_FILE_TABLE_NAME: &str = r#"server"#;
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

        let appender: super::Appender = toml::from_str(&config_toml).unwrap();
        assert!(matches!(appender, Appender::Console(_)));
    }

    #[test]
    fn test_deserialize_rolling_file_appender_toml() {
        let config_toml = format!(
            "{level}{kind}{config}",
            level = DEBUG_LEVEL_TOML,
            kind = ROLLING_FILE_KIND_TOML,
            config = ROLLING_FILE_CONFIG_TOML
        );

        let config: super::Appender = toml::from_str(&config_toml).unwrap();
        assert!(matches!(config, Appender::RollingFile(_)));
    }

    #[test]
    fn test_deserializing_configs_toml() {
        let config_toml = format!(
            "[{console_table}]\n{level}{console_kind}{console_config}[{rolling_file_table}]\n{level}{rolling_file_kind}{rolling_file_config}",
            level = DEBUG_LEVEL_TOML,
            console_table = CONSOLE_TABLE_NAME,
            console_kind = CONSOLE_KIND_TOML,
            console_config = CONSOLE_CONFIG_TOML,
            rolling_file_table = ROLLING_FILE_TABLE_NAME,
            rolling_file_kind = ROLLING_FILE_KIND_TOML,
            rolling_file_config = ROLLING_FILE_CONFIG_TOML
        );

        let configs = toml::from_str::<Configs>(&config_toml).unwrap();
        assert_eq!(configs.appenders.len(), 2);

        assert!(configs.appenders.contains_key(CONSOLE_TABLE_NAME));
        assert!(configs.appenders.contains_key(ROLLING_FILE_TABLE_NAME));

        let console_appender = configs.appenders.get(CONSOLE_TABLE_NAME).unwrap();
        assert!(matches!(console_appender, Appender::Console(_)));
        let rolling_file_appender = configs.appenders.get(ROLLING_FILE_TABLE_NAME).unwrap();
        assert!(matches!(rolling_file_appender, Appender::RollingFile(_)));
    }
}
