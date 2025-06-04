use serde::Deserialize;

use crate::logging::appender::AppenderConfig;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct ConsoleAppenderConfig;

impl AppenderConfig for ConsoleAppenderConfig {
    fn create_appender(&self) -> Result<impl std::io::Write + Send + 'static, crate::error::log_config::LogConfigError> {
        Ok(std::io::stdout())
    }
}