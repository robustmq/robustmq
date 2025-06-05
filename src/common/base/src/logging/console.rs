use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::registry::LookupSpan;

use crate::{
    error::log_config::LogConfigError,
    logging::{
        config::{BoxedLayer, Level},
        fmt::FmtLayerConfig,
    },
};

use super::config::AppenderConfig;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct ConsoleAppenderConfig {
    // Changing this to a unit struct (one without {}) may cause toml
    // deserialization to fail
    level: Level,

    #[serde(flatten)]
    fmt: FmtLayerConfig,
}

impl<S> AppenderConfig<S> for ConsoleAppenderConfig
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn create_layer_and_guard(
        &self,
    ) -> Result<(BoxedLayer<S>, Option<WorkerGuard>), LogConfigError> {
        let writer = std::io::stdout();
        let (non_blocking, guard) = tracing_appender::non_blocking(writer);
        let fmt_layer = self.fmt.create_layer(non_blocking, self.level);

        Ok((fmt_layer, Some(guard)))
    }
}

#[cfg(test)]
mod tests {
    use crate::logging::fmt::Formatter;

    use super::*;

    #[test]
    fn test_deserialize_console_appender_config_default_fmt() {
        let toml_str = r#"
            level = "Debug"
            kind = "Console"
            "#;

        let config: ConsoleAppenderConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.level, Level::Debug);

        assert!(config.fmt.ansi.is_none());
        assert!(config.fmt.formatter.is_none());
    }

    #[test]
    fn test_deserialize_console_appender_config_custom_fmt() {
        let toml_str = r#"
            level = "Info"
            kind = "Console"
            ansi = true
            formatter = "Pretty"
            "#;

        let config: ConsoleAppenderConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.level, Level::Info);
        assert_eq!(config.fmt.ansi, Some(true));
        assert_eq!(config.fmt.formatter, Some(Formatter::Pretty));
    }
}
