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
    cache::{mqtt::MqttCacheManager, placement::PlacementCacheManager},
    storage::rocksdb::RocksDBEngine,
};
use clients::poll::ClientPool;
use dashmap::DashMap;
use message_expire::MessageExpire;
use session_expire::SessionExpire;
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, time::sleep};

pub mod call_broker;
pub mod message_expire;
pub mod session_expire;

pub struct MQTTController {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_center_cache: Arc<PlacementCacheManager>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    client_poll: Arc<ClientPool>,
    thread_running_info: DashMap<String, bool>,
    stop_send: broadcast::Sender<bool>,
}

impl MQTTController {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_center_cache: Arc<PlacementCacheManager>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
        client_poll: Arc<ClientPool>,
        stop_send: broadcast::Sender<bool>,
    ) -> MQTTController {
        return MQTTController {
            rocksdb_engine_handler,
            placement_center_cache,
            mqtt_cache_manager,
            client_poll,
            thread_running_info: DashMap::with_capacity(2),
            stop_send,
        };
    }

    pub async fn start(&self) {
        let mut stop_recv = self.stop_send.subscribe();
        loop {
            select! {
                val = stop_recv.recv() =>{
                    match val{
                        Ok(flag) => {
                            if flag {
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
                _ = self.check_start_thread() => {

                }
            }
        }
    }

    pub async fn check_start_thread(&self) {
        for (cluster_name, _) in self.placement_center_cache.cluster_list.clone() {
            if self.thread_running_info.contains_key(&cluster_name) {
                sleep(Duration::from_secs(5)).await;
                continue;
            }
            // Periodically check if the session has expired
            let session = SessionExpire::new(
                self.rocksdb_engine_handler.clone(),
                self.mqtt_cache_manager.clone(),
                self.placement_center_cache.clone(),
                self.client_poll.clone(),
                cluster_name.clone(),
            );
            let mut stop_recv = self.stop_send.subscribe();
            tokio::spawn(async move {
                loop {
                    select! {
                        val = stop_recv.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {
                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }

                        _ = session.session_expire() =>{
                        }
                    }
                }
            });

            // Periodically check if the session has expired
            let session = SessionExpire::new(
                self.rocksdb_engine_handler.clone(),
                self.mqtt_cache_manager.clone(),
                self.placement_center_cache.clone(),
                self.client_poll.clone(),
                cluster_name.clone(),
            );
            let mut stop_recv = self.stop_send.subscribe();
            tokio::spawn(async move {
                loop {
                    select! {
                        val = stop_recv.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {

                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        _= session.lastwill_expire_send() => {
                        }
                    }
                }
            });

            // Whether the timed message expires
            let message =
                MessageExpire::new(cluster_name.clone(), self.rocksdb_engine_handler.clone());
            let mut stop_recv = self.stop_send.subscribe();
            tokio::spawn(async move {
                loop {
                    select! {
                        val = stop_recv.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {

                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        _ =  message.retain_message_expire() =>{

                        }
                    }
                }
            });

            // Periodically detects whether a will message is sent
            let message =
                MessageExpire::new(cluster_name.clone(), self.rocksdb_engine_handler.clone());
            let mut stop_recv = self.stop_send.subscribe();
            tokio::spawn(async move {
                loop {
                    select! {
                        val = stop_recv.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {

                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        _ = message.last_will_message_expire() => {

                        }
                    }
                }
            });
            self.thread_running_info.insert(cluster_name.clone(), true);
        }
        sleep(Duration::from_secs(5)).await;
    }
}
