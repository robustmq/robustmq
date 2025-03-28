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
use std::time::Duration;

use crate::core::cache::PlacementCacheManager;
use crate::mqtt::cache::MqttCacheManager;
use crate::storage::keys::storage_key_mqtt_session_cluster_prefix;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::session::MqttSessionStorage;
use crate::storage::rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER};
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use grpc_clients::mqtt::inner::call::{broker_mqtt_delete_session, send_last_will_message};
use grpc_clients::pool::ClientPool;
use log::{debug, error, warn};
use metadata_struct::mqtt::lastwill::LastWillData;
use metadata_struct::mqtt::session::MqttSession;
use protocol::broker_mqtt::broker_mqtt_inner::{DeleteSessionRequest, SendLastWillMessageRequest};
use rocksdb_engine::warp::StorageDataWrap;
use tokio::time::sleep;

#[derive(Clone, Debug)]
pub struct ExpireLastWill {
    pub client_id: String,
    pub delay_sec: u64,
    pub cluster_name: String,
}

pub struct SessionExpire {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_pool: Arc<ClientPool>,
    cluster_name: String,
}

impl SessionExpire {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
        placement_cache_manager: Arc<PlacementCacheManager>,
        client_pool: Arc<ClientPool>,
        cluster_name: String,
    ) -> Self {
        SessionExpire {
            rocksdb_engine_handler,
            mqtt_cache_manager,
            placement_cache_manager,
            client_pool,
            cluster_name,
        }
    }

    pub async fn session_expire(&self) {
        let sessions = self.get_expire_session_list().await;
        if !sessions.is_empty() {
            self.delete_session(sessions);
        }
        sleep(Duration::from_secs(1)).await;
    }

    pub async fn lastwill_expire_send(&self) {
        let lastwill_list = self
            .mqtt_cache_manager
            .get_expire_last_wills(&self.cluster_name);

        if !lastwill_list.is_empty() {
            debug!("Will message due, list:{:?}", lastwill_list);
            self.send_expire_lastwill_message(lastwill_list).await;
        }

        sleep(Duration::from_secs(1)).await;
    }

    async fn get_expire_session_list(&self) -> Vec<MqttSession> {
        let search_key = storage_key_mqtt_session_cluster_prefix(&self.cluster_name);
        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_CLUSTER)
        {
            cf
        } else {
            error!(
                "{}",
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_CLUSTER.to_string())
            );
            return Vec::new();
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(search_key.clone());
        let mut sessions = Vec::new();
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            if key.is_none() || value.is_none() {
                iter.next();
                continue;
            }

            let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                Ok(s) => s,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !result_key.starts_with(&search_key) {
                break;
            }

            let result_value = value.unwrap();
            let session = match serde_json::from_slice::<StorageDataWrap>(result_value) {
                Ok(data) => match serde_json::from_str::<MqttSession>(&data.data) {
                    Ok(da) => da,
                    Err(e) => {
                        error!(
                            "Session expired, failed to parse Session data, error message :{}",
                            e.to_string()
                        );
                        iter.next();
                        continue;
                    }
                },
                Err(e) => {
                    error!(
                        "Session expired, failed to parse Session data, error message :{},key:{}",
                        e.to_string(),
                        result_key
                    );
                    iter.next();
                    continue;
                }
            };
            if self.is_session_expire(&session) {
                sessions.push(session);
            }
            iter.next();
        }
        sessions
    }

    fn delete_session(&self, sessions: Vec<MqttSession>) {
        let cluster_name = self.cluster_name.clone();
        let placement_cache_manager = self.placement_cache_manager.clone();
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let client_pool = self.client_pool.clone();
        let mqtt_cache_manager = self.mqtt_cache_manager.clone();
        tokio::spawn(async move {
            delete_sessions(
                cluster_name,
                placement_cache_manager,
                rocksdb_engine_handler,
                client_pool,
                mqtt_cache_manager,
                sessions,
            )
            .await;
        });
    }

    async fn send_expire_lastwill_message(&self, last_will_list: Vec<ExpireLastWill>) {
        let lastwill_storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());
        for lastwill in last_will_list {
            match lastwill_storage.get(&self.cluster_name, &lastwill.client_id) {
                Ok(Some(data)) => {
                    send_last_will(
                        self.cluster_name.clone(),
                        self.placement_cache_manager.clone(),
                        self.client_pool.clone(),
                        self.mqtt_cache_manager.clone(),
                        lastwill.client_id.clone(),
                        data,
                    )
                    .await;
                }
                Ok(None) => {
                    self.mqtt_cache_manager
                        .remove_expire_last_will(&self.cluster_name, &lastwill.client_id);
                }
                Err(e) => {
                    sleep(Duration::from_millis(100)).await;
                    error!("{}", e);
                    continue;
                }
            }
        }
    }

    fn is_session_expire(&self, session: &MqttSession) -> bool {
        if session.connection_id.is_none() && session.broker_id.is_none() {
            if let Some(distinct_time) = session.distinct_time {
                if now_second() >= (session.session_expiry + distinct_time) {
                    return true;
                }
            }
        }

        false
    }
}

pub async fn delete_sessions(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    sessions: Vec<MqttSession>,
) {
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

        for addr in placement_cache_manager.get_broker_node_addr_by_cluster(&cluster_name) {
            let request = DeleteSessionRequest {
                client_id: client_ids.clone(),
                cluster_name: cluster_name.clone(),
            };
            match broker_mqtt_delete_session(&client_pool, &[addr], request).await {
                Ok(_) => {}
                Err(e) => {
                    success = false;
                    warn!("{}", e);
                }
            }
        }
        debug!("Session expired call Broker status: {}", success);
        if success {
            let session_storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
            for ms in raw {
                match session_storage.delete(&cluster_name, &ms.client_id) {
                    Ok(()) => {
                        let delay = ms.last_will_delay_interval.unwrap_or_default();
                        debug!(
                            "Save the upcoming will message to the cache with client ID:{}",
                            ms.client_id
                        );
                        mqtt_cache_manager.add_expire_last_will(ExpireLastWill {
                            client_id: ms.client_id.clone(),
                            delay_sec: now_second() + delay,
                            cluster_name: cluster_name.clone(),
                        });
                    }
                    Err(e) => error!("{}", e),
                }
            }
        }
    }
}

pub async fn send_last_will(
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_pool: Arc<ClientPool>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    client_id: String,
    lastwill: LastWillData,
) {
    let request = SendLastWillMessageRequest {
        client_id: client_id.clone(),
        last_will_message: lastwill.encode(),
    };

    let node_addr = placement_cache_manager.get_broker_node_addr_by_cluster(&cluster_name);

    if node_addr.is_empty() {
        debug!("Get cluster {} Node access address is empty, there is no cluster node address available.",cluster_name);
        mqtt_cache_manager.remove_expire_last_will(&cluster_name, &client_id);
        return;
    }

    match send_last_will_message(&client_pool, &node_addr, request).await {
        Ok(_) => mqtt_cache_manager.remove_expire_last_will(&cluster_name, &client_id),
        Err(e) => {
            error!("{}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use common_base::tools::{now_second, unique_id};
    use common_base::utils::file_utils::test_temp_dir;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::session::MqttSession;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::{ExpireLastWill, SessionExpire};
    use crate::core::cache::PlacementCacheManager;
    use crate::mqtt::cache::MqttCacheManager;
    use crate::mqtt::is_send_last_will;
    use crate::storage::mqtt::session::MqttSessionStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[test]
    fn is_session_expire_test() {
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            1000,
            column_family_list(),
        ));
        let placement_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let mqtt_cache_manager = Arc::new(MqttCacheManager::new());
        let client_pool = Arc::new(ClientPool::new(10));

        let session_expire = SessionExpire::new(
            rocksdb_engine_handler,
            mqtt_cache_manager,
            placement_cache,
            client_pool,
            cluster_name,
        );

        let session = MqttSession {
            session_expiry: now_second() - 100,
            distinct_time: Some(5),
            ..Default::default()
        };
        assert!(session_expire.is_session_expire(&session));

        let session = MqttSession {
            broker_id: Some(1),
            reconnect_time: Some(now_second()),
            session_expiry: now_second() + 100,
            distinct_time: None,
            ..Default::default()
        };
        assert!(!session_expire.is_session_expire(&session));
    }

    #[tokio::test]
    async fn get_expire_session_list_test() {
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            1000,
            column_family_list(),
        ));
        let placement_cache = Arc::new(PlacementCacheManager::new(rocksdb_engine_handler.clone()));
        let mqtt_cache_manager = Arc::new(MqttCacheManager::new());
        let client_pool = Arc::new(ClientPool::new(10));

        let session_expire = SessionExpire::new(
            rocksdb_engine_handler.clone(),
            mqtt_cache_manager,
            placement_cache,
            client_pool,
            cluster_name.clone(),
        );

        let session_storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
        let client_id = unique_id();
        let session = MqttSession {
            session_expiry: 3,
            distinct_time: Some(now_second()),
            client_id: client_id.clone(),
            ..Default::default()
        };

        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let start = now_second();
        loop {
            let expire_list = session_expire.get_expire_session_list().await;

            if !expire_list.is_empty() {
                let mut flag = false;
                for st in expire_list {
                    if st.client_id == client_id {
                        flag = true;
                    }
                }

                if flag {
                    break;
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }
        let esp = now_second() - start;
        println!("{}", esp);
        assert!((3..=5).contains(&esp));
    }

    #[tokio::test]
    async fn is_send_last_will_test() {
        let lastwill = ExpireLastWill {
            client_id: unique_id(),
            delay_sec: now_second() - 3,
            cluster_name: "test1".to_string(),
        };

        assert!(is_send_last_will(&lastwill));

        let lastwill = ExpireLastWill {
            client_id: unique_id(),
            delay_sec: now_second() + 3,
            cluster_name: "test1".to_string(),
        };
        assert!(!is_send_last_will(&lastwill));
    }

    #[tokio::test]
    async fn get_expire_lastwill_message_test() {
        let cluster_name = unique_id();
        let mqtt_cache_manager = Arc::new(MqttCacheManager::new());

        let client_id = unique_id();
        let expire_last_will = ExpireLastWill {
            client_id: client_id.clone(),
            delay_sec: now_second() + 3,
            cluster_name: cluster_name.clone(),
        };

        mqtt_cache_manager.add_expire_last_will(expire_last_will);

        let start = now_second();
        loop {
            let lastwill_list = mqtt_cache_manager.get_expire_last_wills(&cluster_name);
            if !lastwill_list.is_empty() {
                let mut flag = false;
                for st in lastwill_list {
                    if st.client_id == client_id {
                        flag = true;
                    }
                }

                if flag {
                    break;
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }

        assert_eq!((now_second() - start), 3);
    }
}
