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
    handler::cache::CacheManager, server::connection_manager::ConnectionManager,
    subscribe::manager::SubscribeManager,
};
use common_base::tools::now_second;
use dashmap::DashMap;
use std::{sync::Arc, time::Duration};
use tokio::{sync::broadcast, time::sleep};

#[derive(Clone, Default)]
pub struct MetricsCacheManager {
    pub connection_num: DashMap<u64, u32>,
    pub topic_num: DashMap<u64, u32>,
    pub subscribe_num: DashMap<u64, u32>,
    pub message_in_num: DashMap<u64, u32>,
    pub message_out_num: DashMap<u64, u32>,
    pub message_drop_num: DashMap<u64, u32>,
}

impl MetricsCacheManager {
    pub fn new() -> Self {
        MetricsCacheManager {
            connection_num: DashMap::with_capacity(4),
            topic_num: DashMap::with_capacity(4),
            subscribe_num: DashMap::with_capacity(4),
            message_in_num: DashMap::with_capacity(4),
            message_out_num: DashMap::with_capacity(4),
            message_drop_num: DashMap::with_capacity(4),
        }
    }

    pub fn record_connection_num(&self, time: u64, num: u32) {
        self.connection_num.insert(time, num);
    }

    pub fn record_topic_num(&self, time: u64, num: u32) {
        self.topic_num.insert(time, num);
    }

    pub fn record_subscribe_num(&self, time: u64, num: u32) {
        self.subscribe_num.insert(time, num);
    }

    pub fn record_message_in_num(&self, time: u64, num: u32) {
        self.message_in_num.insert(time, num);
    }

    pub fn record_message_out_num(&self, time: u64, num: u32) {
        self.message_out_num.insert(time, num);
    }

    pub fn record_message_drop_num(&self, time: u64, num: u32) {
        self.message_drop_num.insert(time, num);
    }

    pub fn get_connection_num_by_time(
        &self,
        _start_time: u64,
        _end_time: u64,
    ) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }

    pub fn get_topic_num_by_time(&self, _start_time: u64, _end_time: u64) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }

    pub fn get_subscribe_num_by_time(&self, _start_time: u64, _end_time: u64) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }

    pub fn get_message_in_num_by_time(
        &self,
        _start_time: u64,
        _end_time: u64,
    ) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }

    pub fn get_message_out_num_by_time(
        &self,
        _start_time: u64,
        _end_time: u64,
    ) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }

    pub fn get_message_drop_num_by_time(
        &self,
        _start_time: u64,
        _end_time: u64,
    ) -> DashMap<u64, u32> {
        let result = DashMap::with_capacity(2);

        result
    }
}

pub fn metrics_record_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let now = now_second();
        if now_second() % 60 == 0 {
            metrics_cache_manager
                .record_connection_num(now, connection_manager.connections.len() as u32);
            metrics_cache_manager.record_topic_num(now, cache_manager.topic_info.len() as u32);
            metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.subscribe_list.len() as u32);
            metrics_cache_manager.record_message_in_num(now, 0);
            metrics_cache_manager.record_message_out_num(now, 0);
            metrics_cache_manager.record_message_drop_num(now, 0);
        }
        sleep(Duration::from_secs(1)).await;
    });
}

pub fn metrics_gc_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async {
        sleep(Duration::from_secs(3600)).await;
    });
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use common_base::tools::now_second;
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn minute_test() {
        let mut times = 0;
        loop {
            if times >= 3 {
                break;
            }
            let now = now_second();
            if now % 60 == 0 {
                println!("{}", now);
                times += 1;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}
