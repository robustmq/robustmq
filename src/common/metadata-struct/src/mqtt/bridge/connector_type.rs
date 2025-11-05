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
use std::str::FromStr;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum ConnectorType {
    #[default]
    Kafka,
    LocalFile,
    GreptimeDB,
    Pulsar,
    Postgres,
    MongoDB,
    RabbitMQ,
    MySQL,
    Elasticsearch,
    Redis,
}

pub const CONNECTOR_TYPE_FILE: &str = "file";
pub const CONNECTOR_TYPE_KAFKA: &str = "kafka";
pub const CONNECTOR_TYPE_GREPTIMEDB: &str = "greptime";
pub const CONNECTOR_TYPE_PULSAR: &str = "pulsar";
pub const CONNECTOR_TYPE_POSTGRES: &str = "postgres";
pub const CONNECTOR_TYPE_MONGODB: &str = "mongodb";
pub const CONNECTOR_TYPE_RABBITMQ: &str = "rabbitmq";
pub const CONNECTOR_TYPE_MYSQL: &str = "mysql";
pub const CONNECTOR_TYPE_ELASTICSEARCH: &str = "elasticsearch";
pub const CONNECTOR_TYPE_REDIS: &str = "redis";

impl ConnectorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectorType::Kafka => CONNECTOR_TYPE_KAFKA,
            ConnectorType::LocalFile => CONNECTOR_TYPE_FILE,
            ConnectorType::GreptimeDB => CONNECTOR_TYPE_GREPTIMEDB,
            ConnectorType::Pulsar => CONNECTOR_TYPE_PULSAR,
            ConnectorType::Postgres => CONNECTOR_TYPE_POSTGRES,
            ConnectorType::MongoDB => CONNECTOR_TYPE_MONGODB,
            ConnectorType::RabbitMQ => CONNECTOR_TYPE_RABBITMQ,
            ConnectorType::MySQL => CONNECTOR_TYPE_MYSQL,
            ConnectorType::Elasticsearch => CONNECTOR_TYPE_ELASTICSEARCH,
            ConnectorType::Redis => CONNECTOR_TYPE_REDIS,
        }
    }
}

impl Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ConnectorType {
    type Err = CommonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case(CONNECTOR_TYPE_FILE) {
            Ok(ConnectorType::LocalFile)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_KAFKA) {
            Ok(ConnectorType::Kafka)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_GREPTIMEDB) {
            Ok(ConnectorType::GreptimeDB)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_PULSAR) {
            Ok(ConnectorType::Pulsar)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_POSTGRES) {
            Ok(ConnectorType::Postgres)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_MONGODB) {
            Ok(ConnectorType::MongoDB)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_RABBITMQ) {
            Ok(ConnectorType::RabbitMQ)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_MYSQL) {
            Ok(ConnectorType::MySQL)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_ELASTICSEARCH) {
            Ok(ConnectorType::Elasticsearch)
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_REDIS) {
            Ok(ConnectorType::Redis)
        } else {
            Err(CommonError::IneligibleConnectorType(s.to_string()))
        }
    }
}
