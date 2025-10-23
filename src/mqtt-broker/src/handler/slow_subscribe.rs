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

use crate::handler::cache::MQTTCacheManager;
use crate::storage::local::LocalStorage;
use crate::subscribe::common::Subscriber;
use common_base::enum_type::delay_type::DelayType;
use common_base::error::ResultCommonError;
use common_base::tools::{get_local_ip, now_second};
use common_config::broker::broker_config;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubscribeData {
    pub subscribe_name: String,
    pub client_id: String,
    pub topic_name: String,
    pub node_info: String,
    pub time_span: u64,
    pub create_time: u64,
}

impl SlowSubscribeData {
    pub fn build(
        subscribe_name: String,
        client_id: String,
        topic_name: String,
        time_span: u64,
    ) -> Self {
        let ip = get_local_ip();
        let node_info = format!("RobustMQ-MQTT@{ip}");
        SlowSubscribeData {
            subscribe_name,
            client_id,
            topic_name,
            time_span,
            node_info,
            create_time: now_second(),
        }
    }
}

pub async fn record_slow_subscribe_data(
    cache_manager: &Arc<MQTTCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    subscriber: &Subscriber,
    send_time: u64,
    record_time: u64,
) -> ResultCommonError {
    if !cache_manager.get_slow_sub_config().await.enable {
        return Ok(());
    }

    let finish_time = now_second();
    let calculate_time = calc_time(send_time, finish_time, record_time);

    if calculate_time <= cache_manager.get_slow_sub_config().await.record_time {
        return Ok(());
    }

    let log = SlowSubscribeData::build(
        subscriber.sub_path.clone(),
        subscriber.client_id.clone(),
        subscriber.topic_name.clone(),
        calculate_time,
    );

    let local_storage = LocalStorage::new(rocksdb_engine_handler.clone());
    local_storage.save_slow_sub_log(log).await?;
    Ok(())
}

fn calc_time(send_time: u64, finish_time: u64, receive_time: u64) -> u64 {
    let broker_config = broker_config();

    let whole_time = finish_time - receive_time;
    let internal_time = send_time - receive_time;
    let response_time = finish_time - send_time;

    match broker_config.get_slow_subscribe_delay_type() {
        DelayType::Whole => whole_time,
        DelayType::Internal => internal_time,
        DelayType::Response => response_time,
    }
}
