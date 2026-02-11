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

use clap::builder::PossibleValue;
use clap::ValueEnum;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum FeatureType {
    SlowSubscribe,
    OfflineMessage,
    SystemAlarm,
    FlappingDetect,
    MqttProtocol,
    MqttSecurity,
    MqttSchema,
}

impl FromStr for FeatureType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}

impl fmt::Display for FeatureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
impl ValueEnum for FeatureType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::SlowSubscribe,
            Self::OfflineMessage,
            Self::SystemAlarm,
            Self::FlappingDetect,
            Self::MqttProtocol,
            Self::MqttSecurity,
            Self::MqttSchema,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            FeatureType::SlowSubscribe => PossibleValue::new("SlowSubscribe"),
            FeatureType::OfflineMessage => PossibleValue::new("OfflineMessage"),
            FeatureType::SystemAlarm => PossibleValue::new("SystemAlarm"),
            FeatureType::FlappingDetect => PossibleValue::new("FlappingDetect"),
            FeatureType::MqttProtocol => PossibleValue::new("MqttProtocol"),
            FeatureType::MqttSecurity => PossibleValue::new("MqttSecurity"),
            FeatureType::MqttSchema => PossibleValue::new("MqttSchema"),
        })
    }
}
