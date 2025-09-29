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
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum ConnectorType {
    #[default]
    Kafka,
    LocalFile,
    GreptimeDB,
    Pulsar,
    Postgres,
}

pub const CONNECTOR_TYPE_FILE: &str = "file";
pub const CONNECTOR_TYPE_KAFKA: &str = "kafka";
pub const CONNECTOR_TYPE_GREPTIMEDB: &str = "greptime";
pub const CONNECTOR_TYPE_PULSAR: &str = "pulsar";
pub const CONNECTOR_TYPE_POSTGRES: &str = "postgres";

impl Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn connector_type_for_string(connector_type: String) -> Result<ConnectorType, CommonError> {
    if CONNECTOR_TYPE_FILE == connector_type {
        return Ok(ConnectorType::LocalFile);
    }

    if CONNECTOR_TYPE_KAFKA == connector_type {
        return Ok(ConnectorType::Kafka);
    }

    if CONNECTOR_TYPE_GREPTIMEDB == connector_type {
        return Ok(ConnectorType::GreptimeDB);
    }

    if CONNECTOR_TYPE_PULSAR == connector_type {
        return Ok(ConnectorType::Pulsar);
    }

    if CONNECTOR_TYPE_POSTGRES == connector_type {
        return Ok(ConnectorType::Postgres);
    }

    Err(CommonError::IneligibleConnectorType(connector_type))
}
