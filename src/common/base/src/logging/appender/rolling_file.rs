use serde::Deserialize;
use tracing_appender::rolling::{InitError, RollingFileAppender};

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

impl RollingFileAppenderConfig {
    pub(super) fn create_appender(&self) -> Result<RollingFileAppender, InitError> {
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
        builder.build(&self.directory)
    }
}