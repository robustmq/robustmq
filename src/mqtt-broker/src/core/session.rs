use std::sync::Arc;
use super::{cache_manager::CacheManager, lastwill_message::last_will_delay_interval};
use crate::storage::session::SessionStorage;
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError, tools::now_second,
};
use metadata_struct::mqtt::session::MQTTSession;
use protocol::mqtt::common::{Connect, ConnectProperties, LastWill, LastWillProperties};

pub async fn save_session(
    connect_id: u64,
    client_id: String,
    connnect: Connect,
    connect_properties: Option<ConnectProperties>,
    last_will: Option<LastWill>,
    last_will_properties: Option<LastWillProperties>,
    client_poll: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
) -> Result<(MQTTSession, bool), RobustMQError> {
    let session_expiry = session_expiry_interval(
        cache_manager.get_cluster_info().session_expiry_interval,
        connect_properties,
    );
    let is_contain_last_will = !last_will.is_none();
    let last_will_delay_interval = last_will_delay_interval(&last_will_properties);

    let (mut session, new_session) = if connnect.clean_session {
        if let Some(data) = cache_manager.get_session_info(&client_id) {
            (data, true)
        } else {
            let session_storage = SessionStorage::new(client_poll.clone());
            match session_storage.get_session(client_id.clone()).await {
                Ok(Some(session)) => (session, false),
                Ok(None) => (
                    MQTTSession::new(
                        client_id.clone(),
                        session_expiry,
                        is_contain_last_will,
                        last_will_delay_interval,
                    ),
                    true,
                ),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    } else {
        (
            MQTTSession::new(
                client_id.clone(),
                session_expiry,
                is_contain_last_will,
                last_will_delay_interval,
            ),
            true,
        )
    };

    let conf = broker_mqtt_conf();
    let session_storage = SessionStorage::new(client_poll);
    if new_session {
        session.update_connnction_id(Some(connect_id));
        session.update_reconnect_time();
        session.update_broker_id(Some(conf.broker_id));
        match session_storage
            .set_session(client_id, session.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        match session_storage
            .update_session(client_id, connect_id, conf.broker_id, now_second(), 0)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }

    return Ok((session, new_session));
}

fn session_expiry_interval(
    cluster_session_expiry_interval: u32,
    connect_properties: Option<ConnectProperties>,
) -> u64 {
    let connection_session_expiry_interval = if let Some(properties) = connect_properties {
        if let Some(ck) = properties.session_expiry_interval {
            ck
        } else {
            cluster_session_expiry_interval
        }
    } else {
        cluster_session_expiry_interval
    };
    let expiry = std::cmp::min(
        cluster_session_expiry_interval,
        connection_session_expiry_interval,
    );
    return expiry as u64;
}
