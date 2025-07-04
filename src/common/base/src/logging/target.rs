use serde::Deserialize;

use crate::logging::config::Level;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(super) struct Target {
    path: String,
    level: Level,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Targets {
    targets: Vec<Target>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum TargetsOrLevel {
    Targets(Targets),
    Level(Level),
}