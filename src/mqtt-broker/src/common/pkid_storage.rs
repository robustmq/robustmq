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

use common_config::broker::broker_config;
use grpc_clients::meta::inner::call::{
    delete_idempotent_data, exists_idempotent_data, set_idempotent_data,
};
use grpc_clients::pool::ClientPool;
use protocol::meta::meta_service_inner::{
    DeleteIdempotentDataRequest, ExistsIdempotentDataRequest, SetIdempotentDataRequest,
};

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;

pub async fn pkid_save(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> ResultMqttBrokerError {
    if cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .client_pkid_persistent
    {
        let conf = broker_config();
        let request = SetIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match set_idempotent_data(client_pool, &conf.get_meta_service_addr(), request).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(MqttBrokerError::CommonError(e.to_string()));
            }
        }
    } else {
        cache_manager.pkid_metadata.add_client_pkid(client_id, pkid);
    }
    Ok(())
}

pub async fn pkid_exists(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> Result<bool, MqttBrokerError> {
    if cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .client_pkid_persistent
    {
        let conf = broker_config();
        let request = ExistsIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match exists_idempotent_data(client_pool, &conf.get_meta_service_addr(), request).await {
            Ok(reply) => Ok(reply.exists),
            Err(e) => Err(MqttBrokerError::CommonError(e.to_string())),
        }
    } else {
        Ok(cache_manager
            .pkid_metadata
            .get_client_pkid(client_id, pkid)
            .is_some())
    }
}

pub async fn pkid_delete(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    client_id: &str,
    pkid: u16,
) -> ResultMqttBrokerError {
    if cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .client_pkid_persistent
    {
        let conf = broker_config();
        let request = DeleteIdempotentDataRequest {
            cluster_name: conf.cluster_name.clone(),
            producer_id: client_id.to_owned(),
            seq_num: pkid as u64,
        };
        match delete_idempotent_data(client_pool, &conf.get_meta_service_addr(), request).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(MqttBrokerError::CommonError(e.to_string()));
            }
        }
    } else {
        cache_manager
            .pkid_metadata
            .delete_client_pkid(client_id, pkid);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::{pkid_delete, pkid_exists, pkid_save};
    use crate::common::tool::test_build_mqtt_cache_manager;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use grpc_clients::pool::ClientPool;

    #[tokio::test]
    #[ignore]
    pub async fn pkid_test() {
        init_broker_conf_by_config(default_broker_config());

        let client_pool = Arc::new(ClientPool::new(2));
        let cache_manager = test_build_mqtt_cache_manager().await;
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
        let mut cluset_info = cache_manager.broker_cache.get_cluster_config().await;
        cluset_info.mqtt_protocol_config.client_pkid_persistent = true;
        cache_manager.broker_cache.set_cluster_config(cluset_info).await;

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
