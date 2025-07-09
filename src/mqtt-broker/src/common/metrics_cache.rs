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
    handler::cache::CacheManager, server::common::connection_manager::ConnectionManager,
    subscribe::manager::SubscribeManager,
};
use common_base::tools::now_second;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, time::sleep};
use tracing::info;

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
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.connection_num.clone(), start_time, end_time)
    }

    pub fn get_topic_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.topic_num.clone(), start_time, end_time)
    }

    pub fn get_subscribe_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.subscribe_num.clone(), start_time, end_time)
    }

    pub fn get_message_in_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_in_num.clone(), start_time, end_time)
    }

    pub fn get_message_out_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_out_num.clone(), start_time, end_time)
    }

    pub fn get_message_drop_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_drop_num.clone(), start_time, end_time)
    }

    fn search_by_time(
        &self,
        data_list: DashMap<u64, u32>,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        let mut results = Vec::new();
        for (time, value) in data_list {
            if time >= start_time && time <= end_time {
                let mut raw = HashMap::new();
                raw.insert("date".to_string(), time);
                raw.insert("value".to_string(), value as u64);
                results.push(raw);
            }
        }
        results
    }
}

pub fn metrics_record_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    let record_func = async move || {
        let now = now_second();
        if now_second() % time_window == 0 {
            metrics_cache_manager
                .record_connection_num(now, connection_manager.connections.len() as u32);
            metrics_cache_manager.record_topic_num(now, cache_manager.topic_info.len() as u32);
            metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.subscribe_list.len() as u32);
            metrics_cache_manager.record_message_in_num(now, 1000);
            metrics_cache_manager.record_message_out_num(now, 1000);
            metrics_cache_manager.record_message_drop_num(now, 30);
        }

        sleep(Duration::from_secs(1)).await;
    };

    info!("Metrics record thread start successfully");
    tokio::spawn(async move {
        let mut sub_thread_stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = sub_thread_stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("Metrics record thread exited successfully");
                            break;
                        }
                    }
                },
                _ = record_func()=> {
                }
            }
        }
    });
}

pub fn metrics_gc_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let save_time = 3600 * 24 * 3;
    let now_time = now_second();
    let record_func = async move || {
        // connection_num
        for (time, _) in metrics_cache_manager.connection_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.connection_num.remove(&time);
            }
        }

        // topic_num
        for (time, _) in metrics_cache_manager.topic_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.topic_num.remove(&time);
            }
        }

        // subscribe_num
        for (time, _) in metrics_cache_manager.subscribe_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.subscribe_num.remove(&time);
            }
        }

        // message_in_num
        for (time, _) in metrics_cache_manager.message_in_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.message_in_num.remove(&time);
            }
        }

        // message_out_num
        for (time, _) in metrics_cache_manager.message_out_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.message_out_num.remove(&time);
            }
        }

        // message_drop_num
        for (time, _) in metrics_cache_manager.message_drop_num.clone() {
            if (time + save_time) < now_time {
                metrics_cache_manager.message_drop_num.remove(&time);
            }
        }

        sleep(Duration::from_secs(3600)).await;
    };
    info!("Metrics gc thread start successfully");
    tokio::spawn(async move {
        let mut sub_thread_stop_rx = stop_send.subscribe();
        loop {
            select! {
                val = sub_thread_stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("Metrics gc thread exited successfully");
                            break;
                        }
                    }
                },
                _ = record_func()=> {
                }
            }
        }
    });
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};

    use common_base::tools::now_second;
    use grpc_clients::pool::ClientPool;
    use tokio::{sync::broadcast, time::sleep};

    use crate::{
        common::metrics_cache::{metrics_gc_thread, metrics_record_thread, MetricsCacheManager},
        handler::cache::CacheManager,
        server::common::connection_manager::ConnectionManager,
        subscribe::manager::SubscribeManager,
    };

    #[tokio::test]
    pub async fn minute_test() {
        let mut times = 0;
        loop {
            if times >= 1 {
                break;
            }
            let now = now_second();
            if now % 60 == 0 {
                println!("{now}");
                times += 1;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    pub async fn metrics_cache_test() {
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let (stop_send, _) = broadcast::channel(2);
        let client_pool = Arc::new(ClientPool::new(100));
        let cluster_name = "test".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));

        let now = now_second();
        metrics_gc_thread(metrics_cache_manager.clone(), stop_send.clone());
        metrics_record_thread(
            metrics_cache_manager.clone(),
            cache_manager,
            subscribe_manager,
            connection_manager,
            1,
            stop_send,
        );

        sleep(Duration::from_secs(10)).await;
        assert_eq!(metrics_cache_manager.connection_num.len(), 10);
        assert_eq!(metrics_cache_manager.topic_num.len(), 10);
        assert_eq!(metrics_cache_manager.subscribe_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_in_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_out_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_drop_num.len(), 10);

        let start_time = now + 2;
        let end_time = now + 8;
        assert_eq!(
            metrics_cache_manager
                .get_connection_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_topic_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_subscribe_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_in_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_out_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_drop_num_by_time(start_time, end_time)
                .len(),
            7
        );
    }
}
