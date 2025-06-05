use serde::Deserialize;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::MakeWriter, registry::LookupSpan, Layer};

use crate::logging::config::{BoxedLayer, Level};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub(super) enum Formatter {
    Compact,
    Pretty,
    Json,
}

// TODO: what else can be customized in the fmt layer?
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub(super) struct FmtLayerConfig {
    pub(super) ansi: Option<bool>,
    pub(super) formatter: Option<Formatter>,
}

impl FmtLayerConfig {
    /// Creates a new Fmt layer with the specified writer and default ANSI setting.
    pub(super) fn create_layer<S, W>(&self, writer: W, level: Level) -> BoxedLayer<S>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
        W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
    {
        let level: tracing::Level = level.into();
        let mut layer = tracing_subscriber::fmt::layer().with_writer(writer);

        let ansi = self.ansi.unwrap_or(true);
        layer = layer.with_ansi(ansi);

        let filter = LevelFilter::from(level);
        match self.formatter {
            Some(Formatter::Compact) => layer.compact().with_filter(filter).boxed(),
            Some(Formatter::Pretty) => layer.pretty().with_filter(filter).boxed(),
            Some(Formatter::Json) => layer.json().with_filter(filter).boxed(),
            None => layer.with_filter(filter).boxed(),
        }
    }
}
