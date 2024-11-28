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

use std::sync::Arc;

use common_base::tools::now_second;
use grpc_clients::mqtt::inner::call::{broker_mqtt_delete_session, send_last_will_message};
use grpc_clients::pool::ClientPool;
use log::{debug, error, warn};
use metadata_struct::mqtt::lastwill::LastWillData;
use metadata_struct::mqtt::session::MqttSession;
use protocol::broker_mqtt::broker_mqtt_inner::{DeleteSessionRequest, SendLastWillMessageRequest};

use super::session_expire::ExpireLastWill;
use crate::core::cache::PlacementCacheManager;
use crate::mqtt::cache::MqttCacheManager;
use crate::storage::mqtt::session::MqttSessionStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct MqttBrokerCall {
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
}

impl MqttBrokerCall {
    pub fn new(
        cluster_name: String,
        placement_cache_manager: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
    ) -> Self {
        MqttBrokerCall {
            cluster_name,
            placement_cache_manager,
            rocksdb_engine_handler,
            client_pool,
            mqtt_cache_manager,
        }
    }

    pub async fn delete_sessions(&self, sessions: Vec<MqttSession>) {
        let chunks: Vec<Vec<MqttSession>> = sessions
            .chunks(100)
            .map(|chunk| chunk.to_vec()) // 将切片转换为Vec
            .collect();

        for raw in chunks {
            let client_ids: Vec<String> = raw.iter().map(|x| x.client_id.clone()).collect();
            let mut success = true;
            debug!(
                "Session [{:?}] has expired. Call Broker to delete the Session information.",
                client_ids
            );

            for addr in self
                .placement_cache_manager
                .get_broker_node_addr_by_cluster(&self.cluster_name)
            {
                let request = DeleteSessionRequest {
                    client_id: client_ids.clone(),
                    cluster_name: self.cluster_name.clone(),
                };
                match broker_mqtt_delete_session(self.client_pool.clone(), &[addr], request).await {
                    Ok(_) => {}
                    Err(e) => {
                        success = false;
                        warn!("{}", e);
                    }
                }
            }
            debug!("Session expired call Broker status: {}", success);
            if success {
                let session_storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
                for ms in raw {
                    match session_storage.delete(&self.cluster_name, &ms.client_id) {
                        Ok(()) => {
                            let delay = ms.last_will_delay_interval.unwrap_or_default();
                            debug!(
                                "Save the upcoming will message to the cache with client ID:{}",
                                ms.client_id
                            );
                            self.mqtt_cache_manager
                                .add_expire_last_will(ExpireLastWill {
                                    client_id: ms.client_id.clone(),
                                    delay_sec: now_second() + delay,
                                    cluster_name: self.cluster_name.clone(),
                                });
                        }
                        Err(e) => error!("{}", e),
                    }
                }
            }
        }
    }

    pub async fn send_last_will_message(&self, client_id: String, lastwill: LastWillData) {
        let request = SendLastWillMessageRequest {
            client_id: client_id.clone(),
            last_will_message: lastwill.encode(),
        };

        let node_addr = self
            .placement_cache_manager
            .get_broker_node_addr_by_cluster(&self.cluster_name);

        if node_addr.is_empty() {
            warn!("Get cluster {} Node access address is empty, there is no cluster node address available.",self.cluster_name);
            self.mqtt_cache_manager
                .remove_expire_last_will(&self.cluster_name, &client_id);
            return;
        }

        match send_last_will_message(self.client_pool.clone(), &node_addr, request).await {
            Ok(_) => self
                .mqtt_cache_manager
                .remove_expire_last_will(&self.cluster_name, &client_id),
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn delete_sessions_test() {}

    #[tokio::test]
    async fn send_last_will_message_test() {}
}
