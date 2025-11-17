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

#[derive(Debug, thiserror::Error)]
pub(crate) enum CodeError {
    #[allow(dead_code)]
    #[error("Invalid code: {0}")]
    InvalidCode(String),

    #[allow(dead_code)]
    #[error("Code conversion error: {0}")]
    CodeConversionError(String),

    #[error("Code length error: expected {0}, got {1}")]
    CodeLengthError(usize, usize),

    #[error("Usize conversion error: value {0} is out of range for {1}")]
    UsizeConversionError(usize, &'static str),

    #[error("UTF-8 decoding error")]
    UTF8DecodingError,

    #[error("Invalid Code: {0} in MQTT Protocol")]
    MQTTInvalidCode(u32),
}
