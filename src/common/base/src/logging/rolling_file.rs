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

use serde::Deserialize;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::registry::LookupSpan;

use crate::{
    error::log_config::LogConfigError,
    logging::{
        config::{AppenderConfig, BoxedLayer},
        fmt::FmtLayerConfig,
    },
};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum Rotation {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl From<Rotation> for tracing_appender::rolling::Rotation {
    fn from(value: Rotation) -> Self {
        match value {
            Rotation::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
            Rotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            Rotation::Daily => tracing_appender::rolling::Rotation::DAILY,
            Rotation::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct RollingFileAppenderConfig {
    rotation: Rotation,
    directory: String,
    prefix: Option<String>,
    suffix: Option<String>,
    max_log_files: Option<usize>,

    #[serde(flatten)]
    fmt: FmtLayerConfig,
}

impl<S> AppenderConfig<S> for RollingFileAppenderConfig
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn create_layer_and_guard(
        self,
    ) -> Result<(BoxedLayer<S>, Option<WorkerGuard>), LogConfigError> {
        let mut builder = tracing_appender::rolling::Builder::new();

        // Optional fields
        if let Some(prefix) = &self.prefix {
            builder = builder.filename_prefix(prefix);
        }
        if let Some(suffix) = &self.suffix {
            builder = builder.filename_suffix(suffix);
        }
        if let Some(max_log_files) = self.max_log_files {
            builder = builder.max_log_files(max_log_files);
        }

        // Mandatory fields
        builder = builder.rotation(self.rotation.into());
        let writer = builder.build(&self.directory)?;

        let (non_blocking, guard) = tracing_appender::non_blocking(writer);
        let fmt_layer = self.fmt.create_layer(non_blocking);
        Ok((fmt_layer, Some(guard)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_rolling_file_appender_config_default_fmt() {
        let toml_str = r#"
            level = "Debug"
            kind = "RollingFile"
            rotation = "Daily"
            directory = "/var/log/myapp"
            prefix = "myapp-"
            suffix = ".log"
            max_log_files = 7
        "#;

        let config: RollingFileAppenderConfig =
            toml::from_str(toml_str).expect("Failed to deserialize config");

        assert_eq!(config.rotation, Rotation::Daily);
        assert_eq!(config.directory, "/var/log/myapp");
        assert_eq!(config.prefix, Some("myapp-".to_string()));
        assert_eq!(config.suffix, Some(".log".to_string()));
        assert_eq!(config.max_log_files, Some(7));
    }

    #[test]
    fn test_deserialize_rolling_file_appender_config_custom_fmt() {
        let toml_str = r#"
            level = "Info"
            kind = "RollingFile"
            rotation = "Hourly"
            directory = "/var/log/myapp"
            prefix = "myapp-"
            suffix = ".log"
            max_log_files = 5
            ansi = true
            formatter = "Pretty"
        "#;

        let config: RollingFileAppenderConfig =
            toml::from_str(toml_str).expect("Failed to deserialize config");

        assert_eq!(config.rotation, Rotation::Hourly);
        assert_eq!(config.directory, "/var/log/myapp");
        assert_eq!(config.prefix, Some("myapp-".to_string()));
        assert_eq!(config.suffix, Some(".log".to_string()));
        assert_eq!(config.max_log_files, Some(5));
        assert_eq!(config.fmt.ansi, Some(true));
        assert_eq!(
            config.fmt.formatter,
            Some(crate::logging::fmt::Formatter::Pretty)
        );
    }
}
