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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    http_response::{error_response, success_response},
    tools::now_second,
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::connector::{
    config_cassandra::CassandraConnectorConfig,
    config_clickhouse::ClickHouseConnectorConfig,
    config_elasticsearch::ElasticsearchConnectorConfig,
    config_greptimedb::GreptimeDBConnectorConfig,
    config_influxdb::InfluxDBConnectorConfig,
    config_kafka::KafkaConnectorConfig,
    config_local_file::LocalFileConnectorConfig,
    config_mongodb::MongoDBConnectorConfig,
    config_mqtt::MqttBridgeConnectorConfig,
    config_mysql::MySQLConnectorConfig,
    config_opentsdb::OpenTSDBConnectorConfig,
    config_postgres::PostgresConnectorConfig,
    config_pulsar::PulsarConnectorConfig,
    config_rabbitmq::RabbitMQConnectorConfig,
    config_redis::RedisConnectorConfig,
    config_s3::S3ConnectorConfig,
    config_webhook::WebhookConnectorConfig,
    connector_type::{
        CONNECTOR_TYPE_CASSANDRA, CONNECTOR_TYPE_CLICKHOUSE, CONNECTOR_TYPE_ELASTICSEARCH,
        CONNECTOR_TYPE_FILE, CONNECTOR_TYPE_GREPTIMEDB, CONNECTOR_TYPE_INFLUXDB,
        CONNECTOR_TYPE_KAFKA, CONNECTOR_TYPE_MONGODB, CONNECTOR_TYPE_MQTT_BRIDGE,
        CONNECTOR_TYPE_MYSQL, CONNECTOR_TYPE_OPENTSDB, CONNECTOR_TYPE_POSTGRES,
        CONNECTOR_TYPE_PULSAR, CONNECTOR_TYPE_RABBITMQ, CONNECTOR_TYPE_REDIS, CONNECTOR_TYPE_S3,
        CONNECTOR_TYPE_WEBHOOK,
    },
    rule::ETLRule,
    status::MQTTStatus,
    ConnectorType, FailureHandlingStrategy, MQTTConnector,
};
use mqtt_broker::storage::connector::ConnectorStorage;
use std::sync::Arc;

use crate::{
    state::HttpState,
    tool::extractor::ValidatedJson,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConnectorListReq {
    pub tenant: Option<String>,
    pub connector_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectorDetailReq {
    pub tenant: String,
    pub connector_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectorDetailResp {
    pub last_send_time: u64,
    pub send_success_total: u64,
    pub send_fail_total: u64,
    pub last_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateConnectorReq {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Connector name length must be between 1-128"
    ))]
    pub connector_name: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Connector type length must be between 1-50"
    ))]
    #[validate(custom(function = "validate_connector_type"))]
    pub connector_type: String,

    #[validate(length(min = 1, max = 4096, message = "Config length must be between 1-4096"))]
    pub config: String,

    pub failure_strategy: FailureStrategy,

    #[validate(length(min = 1, max = 256, message = "Tenant length must be between 1-256"))]
    pub tenant: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate, Default)]
#[validate(schema(function = "validate_failure_strategy"))]
pub struct FailureStrategy {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Failure strategy length must be between 1-128"
    ))]
    pub strategy: String,
    pub retry_total_times: Option<u32>,
    pub wait_time_ms: Option<u64>,
    pub topic_name: Option<String>,
}

fn validate_failure_strategy(strategy: &FailureStrategy) -> Result<(), validator::ValidationError> {
    let strategy_name = strategy.strategy.to_lowercase();
    match strategy_name.as_str() {
        "discard" => Ok(()),
        "discard_after_retry" => {
            if let Some(retry_total_times) = strategy.retry_total_times {
                if retry_total_times == 0 {
                    let mut err = validator::ValidationError::new("invalid_retry_total_times");
                    err.message = Some(std::borrow::Cow::from(
                        "retry_total_times must be greater than 0",
                    ));
                    return Err(err);
                }
            }
            if let Some(wait_time_ms) = strategy.wait_time_ms {
                if wait_time_ms == 0 {
                    let mut err = validator::ValidationError::new("invalid_wait_time_ms");
                    err.message = Some(std::borrow::Cow::from(
                        "wait_time_ms must be greater than 0",
                    ));
                    return Err(err);
                }
            }
            Ok(())
        }
        "dead_message_queue" => {
            if let Some(retry_total_times) = strategy.retry_total_times {
                if retry_total_times == 0 {
                    let mut err = validator::ValidationError::new("invalid_retry_total_times");
                    err.message = Some(std::borrow::Cow::from(
                        "retry_total_times must be greater than 0",
                    ));
                    return Err(err);
                }
            }
            if let Some(wait_time_ms) = strategy.wait_time_ms {
                if wait_time_ms == 0 {
                    let mut err = validator::ValidationError::new("invalid_wait_time_ms");
                    err.message = Some(std::borrow::Cow::from(
                        "wait_time_ms must be greater than 0",
                    ));
                    return Err(err);
                }
            }
            if let Some(topic_name) = &strategy.topic_name {
                let topic_name = topic_name.trim();
                if topic_name.is_empty() {
                    let mut err = validator::ValidationError::new("invalid_dead_letter_topic_name");
                    err.message = Some(std::borrow::Cow::from(
                        "topic_name for dead_message_queue cannot be empty",
                    ));
                    return Err(err);
                }
                if topic_name.len() > 256 {
                    let mut err = validator::ValidationError::new("invalid_dead_letter_topic_name");
                    err.message = Some(std::borrow::Cow::from(
                        "topic_name for dead_message_queue length must be <= 256",
                    ));
                    return Err(err);
                }
            }
            Ok(())
        }
        _ => {
            let mut err = validator::ValidationError::new("invalid_failure_strategy");
            err.message = Some(std::borrow::Cow::from(
                "strategy must be discard, discard_after_retry or dead_message_queue",
            ));
            Err(err)
        }
    }
}

fn validate_connector_type(connector_type: &str) -> Result<(), validator::ValidationError> {
    use std::str::FromStr;
    if ConnectorType::from_str(connector_type).is_ok() {
        return Ok(());
    }
    let mut err = validator::ValidationError::new("invalid_connector_type");
    err.message = Some(std::borrow::Cow::from(
        "Connector type must be kafka, pulsar, rabbitmq, greptime, postgres, mysql, mongodb, \
         elasticsearch, redis, webhook, opentsdb, mqtt, clickhouse, influxdb, cassandra, s3 or file",
    ));
    Err(err)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteConnectorReq {
    #[validate(length(min = 1, max = 64, message = "Tenant length must be between 1-64"))]
    pub tenant: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Connector name length must be between 1-256"
    ))]
    pub connector_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectorListRow {
    pub tenant: String,
    pub connector_name: String,
    pub connector_type: String,
    pub config: String,
    pub topic_name: String,
    pub status: String,
    pub broker_id: String,
    pub create_time: String,
    pub update_time: String,
}

pub async fn connector_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ConnectorListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let filter_tenant = params.tenant.clone();
    let filter_connector_name = params.connector_name.clone();

    let connectors = match (filter_tenant.as_deref(), filter_connector_name.as_deref()) {
        (Some(tenant), Some(name)) => state
            .mqtt_context
            .connector_manager
            .get_connector_by_tenant(tenant, name)
            .into_iter()
            .collect::<Vec<_>>(),
        (Some(tenant), None) => state
            .mqtt_context
            .connector_manager
            .get_connector_by_tenant_list(tenant),
        (None, Some(name)) => state
            .mqtt_context
            .connector_manager
            .get_connector(name)
            .into_iter()
            .collect::<Vec<_>>(),
        (None, None) => state.mqtt_context.connector_manager.get_all_connector(),
    };

    let results: Vec<ConnectorListRow> = connectors
        .into_iter()
        .map(|connector| ConnectorListRow {
            tenant: connector.tenant.clone(),
            connector_name: connector.connector_name.clone(),
            connector_type: connector.connector_type.to_string(),
            config: serde_json::to_string(&connector.connector_type)
                .unwrap_or_else(|_| "{}".to_string()),
            topic_name: connector.topic_name.clone(),
            status: connector.status.to_string(),
            broker_id: connector
                .broker_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            create_time: timestamp_to_local_datetime(connector.create_time as i64),
            update_time: timestamp_to_local_datetime(connector.update_time as i64),
        })
        .collect();
    let filtered = apply_filters(results, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for ConnectorListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant" => Some(self.tenant.clone()),
            "connector_name" => Some(self.connector_name.clone()),
            "connector_type" => Some(self.connector_type.clone()),
            "topic_name" => Some(self.topic_name.clone()),
            "status" => Some(self.status.clone()),
            "broker_id" => Some(self.broker_id.clone()),
            _ => None,
        }
    }
}

pub async fn connector_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateConnectorReq>,
) -> String {
    if let Err(e) = connector_create_inner(&state, params).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn connector_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteConnectorReq>,
) -> String {
    let storage = ConnectorStorage::new(state.client_pool.clone());
    if let Err(e) = storage
        .delete_connector(&params.tenant, &params.connector_name)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}

async fn connector_create_inner(
    state: &Arc<HttpState>,
    params: CreateConnectorReq,
) -> ResultCommonError {
    let connector_type = parse_connector_type(&params.connector_type, &params.config)?;

    let storage = ConnectorStorage::new(state.client_pool.clone());
    let connector = MQTTConnector {
        connector_name: params.connector_name.clone(),
        connector_type,
        failure_strategy: parse_failure_strategy(&params.tenant, params.failure_strategy),
        tenant: params.tenant.clone(),
        topic_name: params.topic_name.clone(),
        status: MQTTStatus::Idle,
        etl_rule: ETLRule::default(),
        broker_id: None,
        create_time: now_second(),
        update_time: now_second(),
    };

    storage.create_connector(connector).await
}

fn parse_connector_type(type_str: &str, config: &str) -> Result<ConnectorType, CommonError> {
    let t = type_str.to_lowercase();
    let connector_type = match t.as_str() {
        CONNECTOR_TYPE_FILE => {
            let c: LocalFileConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::LocalFile(c)
        }
        CONNECTOR_TYPE_KAFKA => {
            let c: KafkaConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Kafka(c)
        }
        CONNECTOR_TYPE_GREPTIMEDB => {
            let c: GreptimeDBConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::GreptimeDB(c)
        }
        CONNECTOR_TYPE_PULSAR => {
            let c: PulsarConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Pulsar(c)
        }
        CONNECTOR_TYPE_POSTGRES => {
            let c: PostgresConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Postgres(c)
        }
        CONNECTOR_TYPE_MONGODB => {
            let c: MongoDBConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::MongoDB(c)
        }
        CONNECTOR_TYPE_RABBITMQ => {
            let c: RabbitMQConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::RabbitMQ(c)
        }
        CONNECTOR_TYPE_MYSQL => {
            let c: MySQLConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::MySQL(c)
        }
        CONNECTOR_TYPE_ELASTICSEARCH => {
            let c: ElasticsearchConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Elasticsearch(c)
        }
        CONNECTOR_TYPE_REDIS => {
            let c: RedisConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Redis(c)
        }
        CONNECTOR_TYPE_WEBHOOK => {
            let c: WebhookConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Webhook(c)
        }
        CONNECTOR_TYPE_OPENTSDB => {
            let c: OpenTSDBConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::OpenTSDB(c)
        }
        CONNECTOR_TYPE_MQTT_BRIDGE => {
            let c: MqttBridgeConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::MqttBridge(c)
        }
        CONNECTOR_TYPE_CLICKHOUSE => {
            let c: ClickHouseConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::ClickHouse(c)
        }
        CONNECTOR_TYPE_INFLUXDB => {
            let c: InfluxDBConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::InfluxDB(c)
        }
        CONNECTOR_TYPE_CASSANDRA => {
            let c: CassandraConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::Cassandra(c)
        }
        CONNECTOR_TYPE_S3 => {
            let c: S3ConnectorConfig = serde_json::from_str(config)?;
            c.validate()?;
            ConnectorType::S3(c)
        }
        _ => return Err(CommonError::IneligibleConnectorType(type_str.to_string())),
    };
    Ok(connector_type)
}

fn parse_failure_strategy(tenant: &str, strategy: FailureStrategy) -> FailureHandlingStrategy {
    use metadata_struct::connector::{DeadMessageQueueStrategy, DiscardAfterRetryStrategy};

    match strategy.strategy.to_lowercase().as_str() {
        "discard" => FailureHandlingStrategy::Discard,
        "discard_after_retry" => {
            let retry_total_times = strategy.retry_total_times.unwrap_or(3);
            let wait_time_ms = strategy.wait_time_ms.unwrap_or(1000);
            FailureHandlingStrategy::DiscardAfterRetry(DiscardAfterRetryStrategy {
                retry_total_times,
                wait_time_ms,
            })
        }
        "dead_message_queue" => {
            let topic_name = strategy
                .topic_name
                .unwrap_or_else(|| "dead_letter_queue".to_string());
            let retry_total_times = strategy.retry_total_times.unwrap_or(3);
            let wait_time_ms = strategy.wait_time_ms.unwrap_or(1000);
            FailureHandlingStrategy::DeadMessageQueue(DeadMessageQueueStrategy {
                tenant: tenant.to_string(),
                topic_name,
                retry_total_times,
                wait_time_ms,
            })
        }
        _ => {
            // Default to Discard if strategy is not recognized
            FailureHandlingStrategy::Discard
        }
    }
}

pub async fn connector_detail(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ConnectorDetailReq>,
) -> String {
    if state
        .mqtt_context
        .connector_manager
        .get_connector_by_tenant(&params.tenant, &params.connector_name)
        .is_none()
    {
        return error_response(format!(
            "Connector {} does not exist.",
            params.connector_name
        ));
    }

    match state
        .mqtt_context
        .connector_manager
        .get_connector_thread(&params.connector_name)
    {
        Some(data) => {
            let req = ConnectorDetailResp {
                last_msg: data.last_msg,
                last_send_time: data.last_send_time,
                send_fail_total: data.send_fail_total,
                send_success_total: data.send_success_total,
            };
            success_response(req)
        }
        None => error_response(format!(
            "Connector thread {} does not exist.",
            params.connector_name
        )),
    }
}
