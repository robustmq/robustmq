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

use clap::builder::PossibleValue;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy)]
pub enum TopicRewriteActionEnum {
    All,
    Publish,
    Subscribe,
}

impl fmt::Display for TopicRewriteActionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl ValueEnum for TopicRewriteActionEnum {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::All, Self::Publish, Self::Subscribe]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            TopicRewriteActionEnum::All => PossibleValue::new("All"),
            TopicRewriteActionEnum::Publish => PossibleValue::new("Publish"),
            TopicRewriteActionEnum::Subscribe => PossibleValue::new("Subscribe"),
        })
    }
}
