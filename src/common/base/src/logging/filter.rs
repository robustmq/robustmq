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
use tracing::{level_filters::LevelFilter, Subscriber};
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::logging::config::BoxedLayer;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(super) enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for LevelFilter {
    fn from(level: Level) -> Self {
        match level {
            Level::Off => LevelFilter::OFF,
            Level::Error => LevelFilter::ERROR,
            Level::Warn => LevelFilter::WARN,
            Level::Info => LevelFilter::INFO,
            Level::Debug => LevelFilter::DEBUG,
            Level::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct Target {
    pub(super) path: String,
    pub(super) level: Level,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Filter {
    Target(Target),

    /// Multiple targets with their respective level filters
    Targets(Vec<Target>),

    /// A single level filter without a specific target
    Level(Level),
}

impl Filter {
    /// Append the filter to the provided layer and return a boxed layer.
    pub(super) fn append_and_box<S, L>(self, layer: L) -> BoxedLayer<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
        L: Layer<S> + Send + Sync + 'static,
    {
        match self {
            Filter::Target(target) => {
                let filter = tracing_subscriber::filter::Targets::new()
                    .with_target(target.path, target.level);
                layer.with_filter(filter).boxed()
            }
            Filter::Targets(targets) => {
                let mut filter = tracing_subscriber::filter::Targets::new();
                for target in targets {
                    filter = filter.with_target(target.path, target.level);
                }
                layer.with_filter(filter).boxed()
            }
            Filter::Level(level) => layer.with_filter(LevelFilter::from(level)).boxed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logging::filter::Level;

    #[test]
    fn test_deserialize_target_from_toml() {
        let toml_str = r#"
            path = "myapp"
            level = "debug"
        "#;

        let target: super::Target = toml::from_str(toml_str).unwrap();
        assert_eq!(target.path, "myapp");
        assert_eq!(target.level, Level::Debug);
    }

    #[test]
    fn test_deserialize_filter_target() {
        let toml_str = r#"
            target = { path = "myapp", level = "info" }
        "#;

        let filter: super::Filter = toml::from_str(toml_str).unwrap();
        if let super::Filter::Target(target) = filter {
            assert_eq!(target.path, "myapp");
            assert_eq!(target.level, Level::Info);
        } else {
            panic!("Expected Filter::Target variant");
        }
    }

    #[test]
    fn test_deserialize_filter_targets() {
        let toml_str = r#"
            targets = [
                { path = "myapp", level = "debug" },
                { path = "myservice", level = "info" }
            ]
        "#;

        let filter: super::Filter = toml::from_str(toml_str).unwrap();
        if let super::Filter::Targets(targets) = filter {
            assert_eq!(targets.len(), 2);
            assert_eq!(targets[0].path, "myapp");
            assert_eq!(targets[0].level, Level::Debug);
            assert_eq!(targets[1].path, "myservice");
            assert_eq!(targets[1].level, Level::Info);
        } else {
            panic!("Expected Filter::Targets variant");
        }
    }

    #[test]
    fn test_deserialize_filter_level() {
        let toml_str = r#"
            level = "warn"
        "#;

        let filter: super::Filter = toml::from_str(toml_str).unwrap();
        if let super::Filter::Level(level) = filter {
            assert_eq!(level, Level::Warn);
        } else {
            panic!("Expected Filter::Level variant");
        }
    }
}
