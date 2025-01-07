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

use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use clap::builder::PossibleValue;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl FromStr for TimeUnit {
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

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
impl ValueEnum for TimeUnit {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Minutes, Self::Hours, Self::Days]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            TimeUnit::Minutes => PossibleValue::new("minutes"),
            TimeUnit::Hours => PossibleValue::new("hours"),
            TimeUnit::Days => PossibleValue::new("days"),
            TimeUnit::Seconds => PossibleValue::new("seconds"),
        })
    }
}
