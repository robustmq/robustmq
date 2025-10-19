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

use crate::{
    extractor::ValidatedJson,
    state::HttpState,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectorListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
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

    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

fn validate_connector_type(connector_type: &str) -> Result<(), validator::ValidationError> {
    match connector_type {
        "kafka" | "pulsar" | "rabbitmq" | "greptime" | "postgres" | "mysql" | "mongodb"
        | "file" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_connector_type");
            err.message = Some(std::borrow::Cow::from("Connector type must be kafka, pulsar, rabbitmq, greptime, postgres, mysql, mongodb or file"));
            Err(err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteConnectorReq {
    #[validate(length(min = 1, max = 256, message = "Connector name length must be between 1-256"))]
    pub connector_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectorListRow {
    pub connector_name: String,
    pub connector_type: String,
    pub config: String,
    pub topic_name: String,
    pub status: String,
    pub broker_id: String,
    pub create_time: String,
    pub update_time: String,
}

use common_base::{
    error::ResultCommonError,
    http_response::{error_response, success_response},
    tools::now_second,
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::mqtt::bridge::{
    config_greptimedb::GreptimeDBConnectorConfig, config_kafka::KafkaConnectorConfig,
    config_local_file::LocalFileConnectorConfig, config_mongodb::MongoDBConnectorConfig,
    config_mysql::MySQLConnectorConfig, config_postgres::PostgresConnectorConfig,
    config_pulsar::PulsarConnectorConfig, config_rabbitmq::RabbitMQConnectorConfig,
    connector::MQTTConnector, connector_type::ConnectorType, status::MQTTStatus,
};
use mqtt_broker::storage::connector::ConnectorStorage;
use std::{str::FromStr, sync::Arc};

pub async fn connector_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ConnectorListReq>,
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

    let mut connectors = Vec::new();
    for connector in state.mqtt_context.connector_manager.get_all_connector() {
        connectors.push(ConnectorListRow {
            connector_name: connector.connector_name.clone(),
            connector_type: connector.connector_type.to_string(),
            config: connector.config.clone(),
            topic_name: connector.topic_name.clone(),
            status: connector.status.to_string(),
            broker_id: if let Some(id) = connector.broker_id {
                id.to_string()
            } else {
                "-".to_string()
            },
            create_time: timestamp_to_local_datetime(connector.create_time as i64),
            update_time: timestamp_to_local_datetime(connector.update_time as i64),
        });
    }

    let filtered = apply_filters(connectors, &options);
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
        .delete_connector(&state.broker_cache.cluster_name, &params.connector_name)
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
    let connector_type = ConnectorType::from_str(&params.connector_type)?;
    connector_config_validator(&connector_type, &params.config)?;

    let storage = ConnectorStorage::new(state.client_pool.clone());
    let connector = MQTTConnector {
        cluster_name: state.broker_cache.cluster_name.clone(),
        connector_name: params.connector_name.clone(),
        connector_type,
        config: params.config.clone(),
        topic_name: params.topic_name.clone(),
        status: MQTTStatus::Idle,
        broker_id: None,
        create_time: now_second(),
        update_time: now_second(),
    };

    storage.create_connector(connector).await
}

fn connector_config_validator(connector_type: &ConnectorType, config: &str) -> ResultCommonError {
    match connector_type {
        ConnectorType::LocalFile => {
            let _file_config: LocalFileConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::Kafka => {
            let _kafka_config: KafkaConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::GreptimeDB => {
            let _greptime_config: GreptimeDBConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::Pulsar => {
            let _pulsar_config: PulsarConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::Postgres => {
            let _postgres_config: PostgresConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::MongoDB => {
            let _mongo_config: MongoDBConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::RabbitMQ => {
            let _rabbitmq_config: RabbitMQConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::MySQL => {
            let _mysql_config: MySQLConnectorConfig = serde_json::from_str(config)?;
        }
    }
    Ok(())
}
