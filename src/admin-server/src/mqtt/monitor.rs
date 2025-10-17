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

use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use dashmap::DashMap;

use crate::{request::mqtt::MonitorDataReq, state::HttpState};

pub enum MonitorDataType {
    ConnectionNum,
    TopicNum,
    SubscribeNum,
    MessageInNum,
    MessageOutNum,
    MessageDropNum,
    TopicInNum,
    TopicOutNum,
    SubscribeSendNum,
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
            "subscribe_send_num" => Ok(MonitorDataType::SubscribeSendNum),
            _ => Err(format!("Unknown monitor data type: {}", s)),
        }
    }
}

pub async fn monitor_data(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<MonitorDataReq>,
) -> String {
    let data_type = match MonitorDataType::from_str(&params.data_type) {
        Ok(data) => data,
        Err(e) => {
            return error_response(e);
        }
    };

    let data = match data_type {
        MonitorDataType::ConnectionNum => state.mqtt_context.metrics_manager.get_connection_num(),
        MonitorDataType::TopicNum => state.mqtt_context.metrics_manager.get_topic_num(),
        MonitorDataType::SubscribeNum => state.mqtt_context.metrics_manager.get_subscribe_num(),
        MonitorDataType::MessageInNum => state.mqtt_context.metrics_manager.get_message_in_num(),
        MonitorDataType::MessageOutNum => state.mqtt_context.metrics_manager.get_message_out_num(),

        MonitorDataType::MessageDropNum => {
            state.mqtt_context.metrics_manager.get_message_drop_num()
        }

        MonitorDataType::TopicInNum => {
            if let Some(topic_name) = params.topic_name {
                state
                    .mqtt_context
                    .metrics_manager
                    .get_topic_in_num(&topic_name)
            } else {
                DashMap::new()
            }
        }

        MonitorDataType::TopicOutNum => {
            if let Some(topic_name) = params.topic_name {
                state
                    .mqtt_context
                    .metrics_manager
                    .get_topic_out_num(&topic_name)
            } else {
                DashMap::new()
            }
        }

        MonitorDataType::SubscribeSendNum => DashMap::new(),
    };

    let resp = state
        .mqtt_context
        .metrics_manager
        .convert_monitor_data(data);

    success_response(resp)
}
