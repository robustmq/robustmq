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
    request::mqtt::{ConnectorListReq, CreateConnectorReq, DeleteConnectorReq},
    response::{mqtt::ConnectorListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{
    error::ResultCommonError,
    http_response::{error_response, success_response},
    tools::now_second,
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::mqtt::bridge::{
    config_greptimedb::GreptimeDBConnectorConfig,
    config_kafka::KafkaConnectorConfig,
    config_local_file::LocalFileConnectorConfig,
    config_postgres::PostgresConnectorConfig,
    config_pulsar::PulsarConnectorConfig,
    connector::MQTTConnector,
    connector_type::{connector_type_for_string, ConnectorType},
    status::MQTTStatus,
};
use mqtt_broker::storage::connector::ConnectorStorage;
use std::sync::Arc;

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
            topic_id: connector.topic_id.clone(),
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
            "topic_id" => Some(self.topic_id.clone()),
            "status" => Some(self.status.clone()),
            "broker_id" => Some(self.broker_id.clone()),
            _ => None,
        }
    }
}

pub async fn connector_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateConnectorReq>,
) -> String {
    if let Err(e) = connector_create_inner(&state, params).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn connector_delete(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<DeleteConnectorReq>,
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
    let connector_type = connector_type_for_string(params.connector_type.clone())?;
    connector_config_validator(&connector_type, &params.config)?;

    let storage = ConnectorStorage::new(state.client_pool.clone());
    let connector = MQTTConnector {
        cluster_name: state.broker_cache.cluster_name.clone(),
        connector_name: params.connector_name.clone(),
        connector_type,
        config: params.config.clone(),
        topic_id: params.topic_id.clone(),
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
    }
    Ok(())
}
