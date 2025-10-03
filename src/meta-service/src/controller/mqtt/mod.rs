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

use crate::controller::mqtt::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use message_expire::MessageExpire;
use rocksdb_engine::RocksDBEngine;
use session_expire::SessionExpire;
use std::sync::Arc;
use tokio::sync::broadcast;

pub mod call_broker;
pub mod connector;
pub mod message_expire;
pub mod session_expire;

pub fn is_send_last_will(lastwill: &ExpireLastWill) -> bool {
    if now_second() >= lastwill.delay_sec {
        return true;
    }
    false
}

pub struct MqttController {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    thread_running_info: DashMap<String, bool>,
    stop_send: broadcast::Sender<bool>,
}

impl MqttController {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<CacheManager>,
        client_pool: Arc<ClientPool>,
        stop_send: broadcast::Sender<bool>,
    ) -> MqttController {
        MqttController {
            rocksdb_engine_handler,
            cache_manager,
            client_pool,
            thread_running_info: DashMap::with_capacity(2),
            stop_send,
        }
    }

    pub async fn start(&self) {
        let ac_fn = async || -> ResultCommonError {
            self.check_start_thread().await;
            Ok(())
        };
        loop_select_ticket(ac_fn, 1, &self.stop_send).await;
    }

    pub async fn check_start_thread(&self) {
        for cluster_name in self.cache_manager.get_all_cluster_name() {
            if self.thread_running_info.contains_key(&cluster_name) {
                continue;
            }
            // Periodically check if the session has expired
            let session = SessionExpire::new(
                self.rocksdb_engine_handler.clone(),
                self.cache_manager.clone(),
                self.client_pool.clone(),
                cluster_name.clone(),
            );
            let ac_fn = async || -> ResultCommonError {
                session.session_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 60, &self.stop_send).await;

            // Periodically check if the session has expired
            let session = SessionExpire::new(
                self.rocksdb_engine_handler.clone(),
                self.cache_manager.clone(),
                self.client_pool.clone(),
                cluster_name.clone(),
            );
            let ac_fn = async || -> ResultCommonError {
                session.last_will_expire_send().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 60, &self.stop_send).await;

            // Whether the timed message expires
            let message =
                MessageExpire::new(cluster_name.clone(), self.rocksdb_engine_handler.clone());
            let ac_fn = async || -> ResultCommonError {
                message.retain_message_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 60, &self.stop_send).await;

            // Periodically detects whether a will message is sent
            let message =
                MessageExpire::new(cluster_name.clone(), self.rocksdb_engine_handler.clone());

            let ac_fn = async || -> ResultCommonError {
                message.last_will_message_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 60, &self.stop_send).await;
            self.thread_running_info.insert(cluster_name.clone(), true);
        }
    }
}
