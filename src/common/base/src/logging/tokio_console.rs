use std::{net::SocketAddr, str::FromStr};

use serde::Deserialize;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::{
    error::log_config::LogConfigError,
    logging::config::{AppenderConfig, BoxedLayer},
};

// TODO: support more advanced configurations
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct TokioConsoleAppenderConfig {
    bind: Option<String>,
}

impl<S> AppenderConfig<S> for TokioConsoleAppenderConfig
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn create_layer_and_guard(
        &self,
    ) -> Result<(BoxedLayer<S>, Option<WorkerGuard>), LogConfigError> {
        let mut builder = console_subscriber::ConsoleLayer::builder();
        if let Some(bind) = &self.bind {
            let socket_addr = SocketAddr::from_str(bind)?;
            builder = builder.server_addr(socket_addr);
        }
        let layer = builder.spawn().boxed();
        Ok((layer, None))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_deserialize_tokio_console_appender_config() {
        let toml_str = r#"
            kind = "TokioConsole"
            bind = "127.0.0.1:6666"
        "#;

        let config: super::TokioConsoleAppenderConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.bind, Some("127.0.0.1:6666".to_string()));
    }
}
