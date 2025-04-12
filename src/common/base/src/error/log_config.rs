#[derive(Debug, thiserror::Error)]
pub enum LogConfigError {
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error("RollingFile appender requires a rotation")]
    RollingFileMissingRotation,

    #[error("RollingFile appender requires a directory")]
    RollingFileMissingDirectory,

    #[error(transparent)]
    RollingFileAppenderInit(#[from] tracing_appender::rolling::InitError),
}