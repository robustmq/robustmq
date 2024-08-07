// Copyright 2023 RobustMQ Team
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


use super::cache_manager::CacheManager;
use clients::{
    placement::placement::call::{
        delete_idempotent_data, exists_idempotent_data, set_idempotent_data,
    },
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use protocol::placement_center::generate::placement::{
    DeleteIdempotentDataRequest, ExistsIdempotentDataRequest, SetIdempotentDataRequest,
};
use std::sync::Arc;

pub async fn pkid_save(
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    client_id: &String,
    pkid: u16,
) -> Result<(), RobustMQError> {
    if cache_manager.get_cluster_info().client_pkid_persistent {
        let conf = broker_mqtt_conf();
        let request = SetIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.clone(),
            seq_num: pkid as u64,
        };
        match set_idempotent_data(client_poll.clone(), conf.placement_center.clone(), request).await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        cache_manager.add_client_pkid(client_id, pkid);
    }
    return Ok(());
}

pub async fn pkid_exists(
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    client_id: &String,
    pkid: u16,
) -> Result<bool, RobustMQError> {
    if cache_manager.get_cluster_info().client_pkid_persistent {
        let conf = broker_mqtt_conf();
        let request = ExistsIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.clone(),
            seq_num: pkid as u64,
        };
        match exists_idempotent_data(client_poll.clone(), conf.placement_center.clone(), request)
            .await
        {
            Ok(reply) => {
                return Ok(reply.exists);
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        return Ok(!cache_manager.get_client_pkid(client_id, pkid).is_none());
    }
}

pub async fn pkid_delete(
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    client_id: &String,
    pkid: u16,
) -> Result<(), RobustMQError> {
    if cache_manager.get_cluster_info().client_pkid_persistent {
        let conf = broker_mqtt_conf();
        let request = DeleteIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.clone(),
            seq_num: pkid as u64,
        };
        match delete_idempotent_data(client_poll.clone(), conf.placement_center.clone(), request)
            .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        cache_manager.delete_client_pkid(client_id, pkid);
    }
    return Ok(());
}

#[cfg(test)]
mod test {
    use super::pkid_delete;
    use super::pkid_exists;
    use super::pkid_save;
    use crate::handler::cache_manager::CacheManager;
    use clients::poll::ClientPool;
    use common_base::{
        config::broker_mqtt::init_broker_mqtt_conf_by_path, log::init_broker_mqtt_log,
    };
    use std::sync::Arc;

    #[tokio::test]
    pub async fn pkid_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);
        init_broker_mqtt_log();
        let cluster_name = "test".to_string();
        let client_poll = Arc::new(ClientPool::new(10));
        let cache_manager = Arc::new(CacheManager::new(client_poll.clone(), cluster_name));
        let client_id = "test".to_string();
        let pkid = 15;
        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(!flag);
            }
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_save(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(()) => {}
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(flag);
            }
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_delete(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(_) => {}
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(!flag);
            }
            Err(_) => {
                assert!(false);
            }
        }
        let mut cluset_info = cache_manager.get_cluster_info();
        cluset_info.client_pkid_persistent = true;
        cache_manager.set_cluster_info(cluset_info);

        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(!flag);
            }
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_save(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(()) => {}
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }

        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(flag);
            }
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_delete(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(_) => {}
            Err(_) => {
                assert!(false);
            }
        }

        match pkid_exists(&cache_manager, &client_poll, &client_id, pkid).await {
            Ok(flag) => {
                assert!(!flag);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
