use serde::Deserialize;

use super::config::AppenderConfig;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct ConsoleAppenderConfig {
    // Changing this to a unit struct (one without {}) may cause toml
    // deserialization to fail
}

impl AppenderConfig for ConsoleAppenderConfig {
    fn create_appender(
        &self,
    ) -> Result<impl std::io::Write + Send + 'static, crate::error::log_config::LogConfigError>
    {
        Ok(std::io::stdout())
    }
}
