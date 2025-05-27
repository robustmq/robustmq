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

use common_base::error::common::CommonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchMarkError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Benchmark execution failed: {0}")]
    ExecutionError(String),

    #[error("An I/O error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parsing error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Common error: {0}")]
    CommonError(#[from] Box<CommonError>),

    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}
