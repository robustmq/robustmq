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
use tracing_subscriber::{fmt::MakeWriter, registry::LookupSpan};

use crate::logging::{
    config::BoxedLayer,
    filter::Filter,
};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Formatter {
    Compact,
    Pretty,
    Json,
}

// TODO: what else can be customized in the fmt layer?
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct FmtLayerConfig {
    #[serde(flatten)]
    pub(super) filter: Filter,
    pub(super) ansi: Option<bool>,
    pub(super) formatter: Option<Formatter>,
}

impl FmtLayerConfig {
    /// Creates a new Fmt layer with the specified writer and default ANSI setting.
    pub(super) fn create_layer<S, W>(self, writer: W) -> BoxedLayer<S>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
        W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
    {
        let mut layer = tracing_subscriber::fmt::layer().with_writer(writer);

        let ansi = self.ansi.unwrap_or(true);
        layer = layer.with_ansi(ansi);

        match self.formatter {
            Some(Formatter::Compact) => self.filter.append_and_box(layer.compact()),
            Some(Formatter::Pretty) => self.filter.append_and_box(layer.pretty()),
            Some(Formatter::Json) => self.filter.append_and_box(layer.json()),
            None => self.filter.append_and_box(layer),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logging::filter::{Level, Target};

    use super::*;

    #[test]
    fn test_deserialize_fmt_confg_level_filter() {
        let toml_str = r#"
            level = "debug"
            ansi = true
            formatter = "pretty"
            "#;

        let config: FmtLayerConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.filter, Filter::Level(Level::Debug));
        assert_eq!(config.ansi, Some(true));
        assert_eq!(config.formatter, Some(Formatter::Pretty));
    }

    #[test]
    fn test_deserialize_fmt_config_target_filter() {
        let toml_str = r#"
            target = { path = "myapp", level = "info" }
            ansi = false
            formatter = "compact"
            "#;

        let config: FmtLayerConfig = toml::from_str(toml_str).unwrap();

        if let Filter::Target(Target { path, level }) = config.filter {
            assert_eq!(path, "myapp");
            assert_eq!(level, Level::Info);
        } else {
            panic!("Expected Filter::Target variant");
        }
    }

    #[test]
    fn test_deserialize_fmt_config_targets_filter() {
        let toml_str = r#"
            targets = [
                { path = "myapp", level = "debug" },
                { path = "myservice", level = "warn" }
            ]
            ansi = true
            formatter = "json"
            "#;

        let config: FmtLayerConfig = toml::from_str(toml_str).unwrap();
        if let Filter::Targets(targets) = config.filter {
            assert_eq!(targets.len(), 2);
            assert_eq!(targets[0].path, "myapp");
            assert_eq!(targets[0].level, Level::Debug);
            assert_eq!(targets[1].path, "myservice");
            assert_eq!(targets[1].level, Level::Warn);
        } else {
            panic!("Expected Filter::Targets variant");
        }
        assert_eq!(config.ansi, Some(true));
        assert_eq!(config.formatter, Some(Formatter::Json));
    }
}
