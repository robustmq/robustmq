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

use crate::connector::{
    config_cassandra::CassandraConnectorConfig, config_clickhouse::ClickHouseConnectorConfig,
    config_elasticsearch::ElasticsearchConnectorConfig,
    config_greptimedb::GreptimeDBConnectorConfig, config_influxdb::InfluxDBConnectorConfig,
    config_kafka::KafkaConnectorConfig, config_local_file::LocalFileConnectorConfig,
    config_mongodb::MongoDBConnectorConfig, config_mqtt::MqttBridgeConnectorConfig,
    config_mysql::MySQLConnectorConfig, config_opentsdb::OpenTSDBConnectorConfig,
    config_postgres::PostgresConnectorConfig, config_pulsar::PulsarConnectorConfig,
    config_rabbitmq::RabbitMQConnectorConfig, config_redis::RedisConnectorConfig,
    config_s3::S3ConnectorConfig, config_webhook::WebhookConnectorConfig,
};

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
pub const CONNECTOR_TYPE_WEBHOOK: &str = "webhook";
pub const CONNECTOR_TYPE_OPENTSDB: &str = "opentsdb";
pub const CONNECTOR_TYPE_MQTT_BRIDGE: &str = "mqtt";
pub const CONNECTOR_TYPE_CLICKHOUSE: &str = "clickhouse";
pub const CONNECTOR_TYPE_INFLUXDB: &str = "influxdb";
pub const CONNECTOR_TYPE_CASSANDRA: &str = "cassandra";
pub const CONNECTOR_TYPE_S3: &str = "s3";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ConnectorType {
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
    Webhook(WebhookConnectorConfig),
    OpenTSDB(OpenTSDBConnectorConfig),
    MqttBridge(MqttBridgeConnectorConfig),
    ClickHouse(ClickHouseConnectorConfig),
    InfluxDB(InfluxDBConnectorConfig),
    Cassandra(CassandraConnectorConfig),
    S3(S3ConnectorConfig),
}

impl Default for ConnectorType {
    fn default() -> Self {
        ConnectorType::Kafka(KafkaConnectorConfig::default())
    }
}

impl ConnectorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectorType::Kafka(_) => CONNECTOR_TYPE_KAFKA,
            ConnectorType::LocalFile(_) => CONNECTOR_TYPE_FILE,
            ConnectorType::GreptimeDB(_) => CONNECTOR_TYPE_GREPTIMEDB,
            ConnectorType::Pulsar(_) => CONNECTOR_TYPE_PULSAR,
            ConnectorType::Postgres(_) => CONNECTOR_TYPE_POSTGRES,
            ConnectorType::MongoDB(_) => CONNECTOR_TYPE_MONGODB,
            ConnectorType::RabbitMQ(_) => CONNECTOR_TYPE_RABBITMQ,
            ConnectorType::MySQL(_) => CONNECTOR_TYPE_MYSQL,
            ConnectorType::Elasticsearch(_) => CONNECTOR_TYPE_ELASTICSEARCH,
            ConnectorType::Redis(_) => CONNECTOR_TYPE_REDIS,
            ConnectorType::Webhook(_) => CONNECTOR_TYPE_WEBHOOK,
            ConnectorType::OpenTSDB(_) => CONNECTOR_TYPE_OPENTSDB,
            ConnectorType::MqttBridge(_) => CONNECTOR_TYPE_MQTT_BRIDGE,
            ConnectorType::ClickHouse(_) => CONNECTOR_TYPE_CLICKHOUSE,
            ConnectorType::InfluxDB(_) => CONNECTOR_TYPE_INFLUXDB,
            ConnectorType::Cassandra(_) => CONNECTOR_TYPE_CASSANDRA,
            ConnectorType::S3(_) => CONNECTOR_TYPE_S3,
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
            Ok(ConnectorType::LocalFile(LocalFileConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_KAFKA) {
            Ok(ConnectorType::Kafka(KafkaConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_GREPTIMEDB) {
            Ok(ConnectorType::GreptimeDB(
                GreptimeDBConnectorConfig::default(),
            ))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_PULSAR) {
            Ok(ConnectorType::Pulsar(PulsarConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_POSTGRES) {
            Ok(ConnectorType::Postgres(PostgresConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_MONGODB) {
            Ok(ConnectorType::MongoDB(MongoDBConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_RABBITMQ) {
            Ok(ConnectorType::RabbitMQ(RabbitMQConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_MYSQL) {
            Ok(ConnectorType::MySQL(MySQLConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_ELASTICSEARCH) {
            Ok(ConnectorType::Elasticsearch(
                ElasticsearchConnectorConfig::default(),
            ))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_REDIS) {
            Ok(ConnectorType::Redis(RedisConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_WEBHOOK) {
            Ok(ConnectorType::Webhook(WebhookConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_OPENTSDB) {
            Ok(ConnectorType::OpenTSDB(OpenTSDBConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_MQTT_BRIDGE) {
            Ok(ConnectorType::MqttBridge(
                MqttBridgeConnectorConfig::default(),
            ))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_CLICKHOUSE) {
            Ok(ConnectorType::ClickHouse(
                ClickHouseConnectorConfig::default(),
            ))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_INFLUXDB) {
            Ok(ConnectorType::InfluxDB(InfluxDBConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_CASSANDRA) {
            Ok(ConnectorType::Cassandra(CassandraConnectorConfig::default()))
        } else if s.eq_ignore_ascii_case(CONNECTOR_TYPE_S3) {
            Ok(ConnectorType::S3(S3ConnectorConfig::default()))
        } else {
            Err(CommonError::IneligibleConnectorType(s.to_string()))
        }
    }
}
