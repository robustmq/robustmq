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

use super::{
    config_elasticsearch::ElasticsearchConnectorConfig, config_greptimedb::GreptimeDBConnectorConfig,
    config_kafka::KafkaConnectorConfig, config_local_file::LocalFileConnectorConfig,
    config_mongodb::MongoDBConnectorConfig, config_mysql::MySQLConnectorConfig,
    config_postgres::PostgresConnectorConfig, config_pulsar::PulsarConnectorConfig,
    config_rabbitmq::RabbitMQConnectorConfig, config_redis::RedisConnectorConfig,
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

