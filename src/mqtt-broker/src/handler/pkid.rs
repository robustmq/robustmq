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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use grpc_clients::placement::inner::call::{
    delete_idempotent_data, exists_idempotent_data, set_idempotent_data,
};
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_inner::{
    DeleteIdempotentDataRequest, ExistsIdempotentDataRequest, SetIdempotentDataRequest,
};

use super::cache::CacheManager;

pub async fn pkid_save(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> Result<(), CommonError> {
    if cache_manager
        .get_cluster_info()
        .protocol
        .client_pkid_persistent
    {
        let conf = broker_mqtt_conf();
        let request = SetIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match set_idempotent_data(client_pool.clone(), &conf.placement_center, request).await {
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
    Ok(())
}

pub async fn pkid_exists(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> Result<bool, CommonError> {
    if cache_manager
        .get_cluster_info()
        .protocol
        .client_pkid_persistent
    {
        let conf = broker_mqtt_conf();
        let request = ExistsIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match exists_idempotent_data(client_pool.clone(), &conf.placement_center, request).await {
            Ok(reply) => Ok(reply.exists),
            Err(e) => Err(e),
        }
    } else {
        Ok(cache_manager.get_client_pkid(client_id, pkid).is_some())
    }
}

pub async fn pkid_delete(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> Result<(), CommonError> {
    if cache_manager
        .get_cluster_info()
        .protocol
        .client_pkid_persistent
    {
        let conf = broker_mqtt_conf();
        let request = DeleteIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match delete_idempotent_data(client_pool.clone(), &conf.placement_center, request).await {
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
    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use grpc_clients::pool::ClientPool;

    use super::{pkid_delete, pkid_exists, pkid_save};
    use crate::handler::cache::CacheManager;

    #[tokio::test]
    #[ignore]
    pub async fn pkid_test() {
        let path = format!(
            "{}/../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_broker_mqtt_conf_by_path(&path);

        let cluster_name = "test".to_string();
        let client_pool = Arc::new(ClientPool::new(10));
        let cache_manager = Arc::new(CacheManager::new(client_pool.clone(), cluster_name));
        let client_id = "test".to_string();
        let pkid = 15;
        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(!flag);

        pkid_save(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();

        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(flag);

        pkid_delete(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();

        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(!flag);
        let mut cluset_info = cache_manager.get_cluster_info();
        cluset_info.protocol.client_pkid_persistent = true;
        cache_manager.set_cluster_info(cluset_info);

        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(!flag);

        pkid_save(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();

        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(flag);

        pkid_delete(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();

        let flag = pkid_exists(&cache_manager, &client_pool, &client_id, pkid)
            .await
            .unwrap();
        assert!(!flag);
    }
}
