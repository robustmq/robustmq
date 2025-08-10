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

use crate::common::metrics_cache::MetricsCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::observability::slow::slow_subscribe_data::SlowSubscribeData;
use crate::observability::slow::slow_subscribe_key::SlowSubscribeKey;
use crate::subscribe::common::Subscriber;
use protocol::broker_mqtt::broker_mqtt_admin::ListSlowSubscribeRequest;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

pub mod slow_subscribe_data;
pub mod slow_subscribe_key;

pub fn record_slow_subscribe_data(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    calculate_time: u64,
    config_num: usize,
    subscriber: &Subscriber,
) {
    let client_id = subscriber.client_id.clone();
    let topic_name = subscriber.topic_name.clone();
    let subscribe_name = subscriber.sub_path.clone();
    // 如果索引当中有值那么就将对应的值拿出来
    let option_slow_subscribe_index_value =
        metrics_cache_manager.get_slow_subscribe_index_value(client_id.clone(), topic_name.clone());

    // 处理新的
    let new_slow_subscribe_info_key = SlowSubscribeKey {
        time_span: calculate_time,
        client_id: client_id.clone(),
        topic_name: topic_name.clone(),
    };

    let slow_subscribe_info_data: SlowSubscribeData = SlowSubscribeData::build(
        subscribe_name.clone(),
        client_id.clone(),
        topic_name.clone(),
        calculate_time,
    );

    if let Some(slow_subscribe_index_value) = option_slow_subscribe_index_value {
        let time_span = slow_subscribe_index_value.0;
        if calculate_time > time_span {
            // 处理旧的
            let old_slow_subscribe_info_key = SlowSubscribeKey {
                time_span,
                client_id: client_id.clone(),
                topic_name: topic_name.clone(),
            };

            metrics_cache_manager.remove_slow_subscribe_info(&old_slow_subscribe_info_key);
        }
    } else {
        // 如果索引当中没有值，那么此时需要对info操作
        if metrics_cache_manager.slow_subscribe_info.len() + 1 > config_num {
            // info此时是否大于1000
            // 这个时候拿出最小的time_span的key，比较对应的time_span值
            let option_slow_subscribe_info_key =
                metrics_cache_manager.slow_subscribe_info.min_key();
            let min_time_span = if let Some(key) = option_slow_subscribe_info_key.clone() {
                key.time_span
            } else {
                0
            };

            if calculate_time > min_time_span {
                // 删除旧的
                if let Some(key) = option_slow_subscribe_info_key {
                    metrics_cache_manager
                        .remove_slow_subscribe_index(key.client_id.clone(), key.topic_name.clone());
                    metrics_cache_manager.remove_slow_subscribe_info(&key);
                }
            }
        }
    }

    // 插入数据
    metrics_cache_manager
        .record_slow_subscribe_info(new_slow_subscribe_info_key, slow_subscribe_info_data);
    metrics_cache_manager.record_slow_subscribe_index(
        client_id.clone(),
        topic_name.clone(),
        calculate_time,
    )
}

pub fn read_slow_sub_record(
    _search_options: ListSlowSubscribeRequest,
    _path: PathBuf,
) -> Result<VecDeque<String>, MqttBrokerError> {
    // 该部分需要重写
    todo!()
}

#[cfg(test)]
mod tests {}
