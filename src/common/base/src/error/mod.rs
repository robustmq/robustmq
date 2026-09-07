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

use crate::error::common::CommonError;

pub mod common;
pub mod log_config;
pub mod mqtt_protocol_error;

pub type ResultCommonError = Result<(), CommonError>;

pub fn client_unavailable_error_by_str(error: &str) -> bool {
    error.contains("Connection management could not obtain an available")
        || error.contains("IO error: Broken pipe")
        || error.contains("Broken pipe (os error 32)")
        || error.contains("Broken pipe")
        || error.contains("work with closed connection")
}
