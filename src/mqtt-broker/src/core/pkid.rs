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
        match set_idempotent_data(client_poll.clone(), conf.placement.server.clone(), request).await
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
        match exists_idempotent_data(client_poll.clone(), conf.placement.server.clone(), request)
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
        return Ok(cache_manager
            .get_client_pkid(client_id, pkid)
            .await
            .is_none());
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
        match delete_idempotent_data(client_poll.clone(), conf.placement.server.clone(), request)
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
