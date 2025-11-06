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

use super::{
    config_elasticsearch::ElasticsearchConnectorConfig,
    config_greptimedb::GreptimeDBConnectorConfig, config_kafka::KafkaConnectorConfig,
    config_local_file::LocalFileConnectorConfig, config_mongodb::MongoDBConnectorConfig,
    config_mysql::MySQLConnectorConfig, config_postgres::PostgresConnectorConfig,
    config_pulsar::PulsarConnectorConfig, config_rabbitmq::RabbitMQConnectorConfig,
    config_redis::RedisConnectorConfig, connector_type::ConnectorType, status::MQTTStatus,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ConnectorConfig {
    Kafka(KafkaConnectorConfig),
    LocalFile(LocalFileConnectorConfig),
    GreptimeDB(GreptimeDBConnectorConfig),
    Pulsar(PulsarConnectorConfig),
    Postgres(PostgresConnectorConfig),
    MongoDB(MongoDBConnectorConfig),
    RabbitMQ(RabbitMQConnectorConfig),
    MySQL(MySQLConnectorConfig),
    Elasticsearch(ElasticsearchConnectorConfig),
    Redis(RedisConnectorConfig),
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        ConnectorConfig::Kafka(KafkaConnectorConfig::default())
    }
}

impl ConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        match self {
            ConnectorConfig::Kafka(config) => config.validate(),
            ConnectorConfig::LocalFile(config) => config.validate(),
            ConnectorConfig::GreptimeDB(config) => config.validate(),
            ConnectorConfig::Pulsar(config) => config.validate(),
            ConnectorConfig::Postgres(config) => config.validate(),
            ConnectorConfig::MongoDB(config) => config.validate(),
            ConnectorConfig::RabbitMQ(config) => config.validate(),
            ConnectorConfig::MySQL(config) => config.validate(),
            ConnectorConfig::Elasticsearch(config) => config.validate(),
            ConnectorConfig::Redis(config) => config.validate(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct MQTTConnector {
    pub cluster_name: String,
    pub connector_name: String,
    pub connector_type: ConnectorType,
    pub config: ConnectorConfig,
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
}

impl MQTTConnector {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }

    pub fn touch(&mut self, timestamp: u64) {
        self.update_time = timestamp;
    }

    pub fn is_assigned(&self) -> bool {
        self.broker_id.is_some()
    }

    pub fn unique_key(&self) -> String {
        format!("{}/{}", self.cluster_name, self.connector_name)
    }

    pub fn validate(&self) -> Result<(), CommonError> {
        if self.cluster_name.is_empty() {
            return Err(CommonError::CommonError(
                "cluster_name cannot be empty".to_string(),
            ));
        }

        if self.connector_name.is_empty() {
            return Err(CommonError::CommonError(
                "connector_name cannot be empty".to_string(),
            ));
        }

        if self.topic_name.is_empty() {
            return Err(CommonError::CommonError(
                "topic_name cannot be empty".to_string(),
            ));
        }

        self.config.validate()
    }
}
