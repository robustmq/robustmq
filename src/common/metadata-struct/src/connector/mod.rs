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

use common_base::{error::common::CommonError, utils::serialize};
use serde::{Deserialize, Serialize};

pub mod config_cassandra;
pub mod config_clickhouse;
pub mod config_elasticsearch;
pub mod config_greptimedb;
pub mod config_influxdb;
pub mod config_kafka;
pub mod config_local_file;
pub mod config_mongodb;
pub mod config_mqtt;
pub mod config_mysql;
pub mod config_opentsdb;
pub mod config_postgres;
pub mod config_pulsar;
pub mod config_rabbitmq;
pub mod config_redis;
pub mod config_s3;
pub mod config_webhook;
pub mod connector_type;
pub mod status;

pub use connector_type::ConnectorType;

use status::MQTTStatus;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct MQTTConnector {
    pub connector_name: String,
    pub connector_type: ConnectorType,
    pub failure_strategy: FailureHandlingStrategy,
    pub topic_name: String,
    pub status: MQTTStatus,
    pub broker_id: Option<u64>,
    pub create_time: u64,
    pub update_time: u64,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum FailureHandlingStrategy {
    #[default]
    Discard,
    DiscardAfterRetry(DiscardAfterRetryStrategy),
    DeadMessageQueue(DeadMessageQueueStrategy),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DiscardAfterRetryStrategy {
    pub retry_total_times: u32,
    pub wait_time_ms: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DeadMessageQueueStrategy {
    pub topic_name: String,
    #[serde(default = "default_retry_total_times")]
    pub retry_total_times: u32,
    #[serde(default = "default_wait_time_ms")]
    pub wait_time_ms: u64,
}

fn default_retry_total_times() -> u32 {
    3
}

fn default_wait_time_ms() -> u64 {
    1000
}

impl MQTTConnector {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
