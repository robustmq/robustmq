use std::io;

use serde::Deserialize;
use tracing_appender::rolling::{InitError, RollingFileAppender};

use crate::{error::log_config::LogConfigError, logging::appender::AppenderConfig};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
}

impl AppenderConfig for RollingFileAppenderConfig {
    fn create_appender(&self) -> Result<impl io::Write + Send + 'static, LogConfigError> {
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
        builder.build(&self.directory).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_rolling_file_appender_config() {
        let toml_str = r#"
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
}