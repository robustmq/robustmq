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

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Query, State};
use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
};
use dashmap::DashMap;
use serde::Deserialize;

use crate::state::HttpState;
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;

#[derive(Deserialize)]
pub struct MonitorDataReq {
    pub data_type: String,
    pub topic_name: Option<String>,
    pub client_id: Option<String>,
    pub path: Option<String>,
    pub connector_name: Option<String>,
}

pub enum MonitorDataType {
    ConnectionNum,
    TopicNum,
    SubscribeNum,
    MessageInNum,
    MessageOutNum,
    MessageDropNum,
    TopicInNum,
    TopicOutNum,
    SubscribeSendSuccessNum,
    SubscribeSendFailureNum,
    SubscribeTopicSendSuccessNum,
    SubscribeTopicSendFailureNum,
    SessionInNum,
    SessionOutNum,
    ConnectorSendSuccessTotal,
    ConnectorSendFailureTotal,
    ConnectorSendSuccess,
    ConnectorSendFailure,
}

impl FromStr for MonitorDataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "connection_num" => Ok(MonitorDataType::ConnectionNum),
            "topic_num" => Ok(MonitorDataType::TopicNum),
            "subscribe_num" => Ok(MonitorDataType::SubscribeNum),
            "message_in_num" => Ok(MonitorDataType::MessageInNum),
            "message_out_num" => Ok(MonitorDataType::MessageOutNum),
            "message_drop_num" => Ok(MonitorDataType::MessageDropNum),
            "topic_in_num" => Ok(MonitorDataType::TopicInNum),
            "topic_out_num" => Ok(MonitorDataType::TopicOutNum),
            "subscribe_send_success_num" => Ok(MonitorDataType::SubscribeSendSuccessNum),
            "subscribe_send_failure_num" => Ok(MonitorDataType::SubscribeSendFailureNum),
            "subscribe_topic_send_success_num" => Ok(MonitorDataType::SubscribeTopicSendSuccessNum),
            "subscribe_topic_send_failure_num" => Ok(MonitorDataType::SubscribeTopicSendFailureNum),
            "session_in_num" => Ok(MonitorDataType::SessionInNum),
            "session_out_num" => Ok(MonitorDataType::SessionOutNum),
            "connector_send_success_total" => Ok(MonitorDataType::ConnectorSendSuccessTotal),
            "connector_send_failure_total" => Ok(MonitorDataType::ConnectorSendFailureTotal),
            "connector_send_success" => Ok(MonitorDataType::ConnectorSendSuccess),
            "connector_send_failure" => Ok(MonitorDataType::ConnectorSendFailure),
            _ => Err(format!("Unknown monitor data type: {}", s)),
        }
    }
}

pub async fn monitor_data(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<MonitorDataReq>,
) -> String {
    let data_type = match MonitorDataType::from_str(&params.data_type) {
        Ok(data) => data,
        Err(e) => {
            return error_response(e);
        }
    };

    let data: DashMap<u64, u64> =
        match get_monitor_data(&state.mqtt_context.metrics_manager, params, data_type) {
            Ok(data) => data,
            Err(e) => {
                return error_response(e.to_string());
            }
        };

    let resp = state
        .mqtt_context
        .metrics_manager
        .convert_monitor_data(data);

    success_response(resp)
}

pub fn get_monitor_data(
    metrics_manager: &Arc<MQTTMetricsCache>,
    params: MonitorDataReq,
    data_type: MonitorDataType,
) -> Result<DashMap<u64, u64>, CommonError> {
    match data_type {
        MonitorDataType::ConnectionNum => metrics_manager.get_connection_num(),
        MonitorDataType::TopicNum => metrics_manager.get_topic_num(),
        MonitorDataType::SubscribeNum => metrics_manager.get_subscribe_num(),
        MonitorDataType::MessageInNum => metrics_manager.get_message_in_num(),
        MonitorDataType::MessageOutNum => metrics_manager.get_message_out_num(),

        MonitorDataType::MessageDropNum => metrics_manager.get_message_drop_num(),

        MonitorDataType::TopicInNum => {
            if let Some(topic_name) = params.topic_name {
                metrics_manager.get_topic_in_num(&topic_name)
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::TopicOutNum => {
            if let Some(topic_name) = params.topic_name {
                metrics_manager.get_topic_out_num(&topic_name)
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::SubscribeSendSuccessNum => {
            if params.client_id.is_some() && params.path.is_some() {
                metrics_manager.get_subscribe_send_num(
                    &params.client_id.unwrap(),
                    &params.path.unwrap(),
                    true,
                )
            } else {
                Ok(DashMap::new())
            }
        }
        MonitorDataType::SubscribeSendFailureNum => {
            if params.client_id.is_some() && params.path.is_some() {
                metrics_manager.get_subscribe_send_num(
                    &params.client_id.unwrap(),
                    &params.path.unwrap(),
                    false,
                )
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::SubscribeTopicSendSuccessNum => {
            if params.client_id.is_some() && params.path.is_some() && params.topic_name.is_some() {
                metrics_manager.get_subscribe_topic_send_num(
                    &params.client_id.unwrap(),
                    &params.path.unwrap(),
                    &params.topic_name.unwrap(),
                    true,
                )
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::SubscribeTopicSendFailureNum => {
            if params.client_id.is_some() && params.path.is_some() && params.topic_name.is_some() {
                metrics_manager.get_subscribe_topic_send_num(
                    &params.client_id.unwrap(),
                    &params.path.unwrap(),
                    &params.topic_name.unwrap(),
                    false,
                )
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::SessionInNum => {
            if params.client_id.is_some() {
                metrics_manager.get_session_in_num(&params.client_id.unwrap())
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::SessionOutNum => {
            if params.client_id.is_some() {
                metrics_manager.get_session_out_num(&params.client_id.unwrap())
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::ConnectorSendSuccessTotal => {
            metrics_manager.get_connector_success_total_num()
        }

        MonitorDataType::ConnectorSendFailureTotal => {
            metrics_manager.get_connector_failure_total_num()
        }

        MonitorDataType::ConnectorSendSuccess => {
            if let Some(connector_name) = params.connector_name {
                metrics_manager.get_connector_success_num(&connector_name)
            } else {
                Ok(DashMap::new())
            }
        }

        MonitorDataType::ConnectorSendFailure => {
            if let Some(connector_name) = params.connector_name {
                metrics_manager.get_connector_failure_num(&connector_name)
            } else {
                Ok(DashMap::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use rocksdb_engine::test::test_rocksdb_instance;

    #[tokio::test]
    async fn test_get_monitor_data() {
        let rocksdb_engine = test_rocksdb_instance();
        let metrics_manager = Arc::new(MQTTMetricsCache::new(rocksdb_engine));
        let time = now_second();

        // Record some data
        metrics_manager.record_connection_num(time, 100).unwrap();

        let params = MonitorDataReq {
            data_type: "connection_num".to_string(),
            topic_name: None,
            client_id: None,
            path: None,
            connector_name: None,
        };

        let data_type = MonitorDataType::from_str(&params.data_type).unwrap();
        let result = get_monitor_data(&metrics_manager, params, data_type).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(*result.get(&time).unwrap(), 100);
    }
}
