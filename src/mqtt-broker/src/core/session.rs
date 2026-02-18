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

use super::cache::MQTTCacheManager;
use super::error::MqttBrokerError;
use super::last_will::last_will_delay_interval;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::session::SessionStorage;
use crate::subscribe::manager::SubscribeManager;
use common_config::broker::broker_config;
use common_metrics::mqtt::session::record_mqtt_session_created;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use protocol::mqtt::common::{
    Connect, ConnectProperties, LastWill, LastWillProperties, MqttProtocol,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct BuildSessionContext {
    pub connect_id: u64,
    pub client_id: String,
    pub connect: Connect,
    pub connect_properties: Option<ConnectProperties>,
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
}

/// Create, restore, or reset the MQTT session during CONNECT handling.
///
/// - MQTT 3.1 / 3.1.1: `connect.clean_session` corresponds to the Clean Session flag.
/// - MQTT 5.0: the same flag corresponds to Clean Start (the session is started fresh when set).
///
/// Behavior:
/// - If `clean_session/clean_start = 1`: delete any existing session state (both remote storage and
///   local caches), then create a new session and return `(session, true)`.
/// - If `clean_session/clean_start = 0`: try to load an existing session from storage. If found,
///   mark it online (update connection/broker IDs, reconnect time, clear distinct_time), persist it,
///   and return `(session, false)`. If not found, create a new session, persist it, and return
///   `(session, true)`.
///
/// The returned boolean indicates whether a new session was created (`true`) or an existing session
/// was resumed (`false`). Callers should typically map this to CONNACK `Session Present` as
/// `session_present = !new_session`.
pub async fn session_process(
    protocol: &MqttProtocol,
    context: BuildSessionContext,
) -> Result<(MqttSession, bool), MqttBrokerError> {
    let session_storage = SessionStorage::new(context.client_pool.clone());
    if context.connect.clean_session {
        // Clean Session = 1
        delete_session_by_local(
            &context.cache_manager,
            &context.subscribe_manager,
            &context.client_id,
        );
        let session = build_new_session(&context).await;
        if protocol.is_mqtt5() {
            save_session(
                session.clone(),
                context.client_id.clone(),
                &context.client_pool,
            )
            .await?;
        } else {
            session_storage
                .delete_session(context.client_id.clone())
                .await?;
        }
        return Ok((session, true));
    }

    // Clean Session = 0
    if let Some(mut session) = session_storage
        .get_session(context.client_id.clone())
        .await?
    {
        let conf = broker_config();
        session.update_connection_id(Some(context.connect_id));
        session.update_broker_id(Some(conf.broker_id));
        session.update_reconnect_time();
        session.distinct_time = None;
        save_session(
            session.clone(),
            context.client_id.clone(),
            &context.client_pool,
        )
        .await?;
        return Ok((session, false));
    }

    let session = build_new_session(&context).await;
    save_session(
        session.clone(),
        context.client_id.clone(),
        &context.client_pool,
    )
    .await?;
    Ok((session, true))
}

pub fn delete_session_by_local(
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
) {
    subscribe_manager.remove_by_client_id(client_id);
    cache_manager.remove_session(client_id);
}

async fn build_new_session(context: &BuildSessionContext) -> MqttSession {
    let session_expiry =
        session_expiry_interval(&context.cache_manager, &context.connect_properties).await;
    let is_contain_last_will = context.last_will.is_some();
    let last_will_delay_interval = last_will_delay_interval(&context.last_will_properties);
    let mut session = MqttSession::new(
        context.client_id.clone(),
        session_expiry,
        is_contain_last_will,
        last_will_delay_interval,
        is_persist_session(&context.client_id),
    );
    let conf = broker_config();
    session.update_connection_id(Some(context.connect_id));
    session.update_broker_id(Some(conf.broker_id));
    session.update_reconnect_time();
    session
}

fn is_persist_session(_client_id: &str) -> bool {
    // todo
    let conf = broker_config();
    conf.mqtt_runtime.durable_sessions_enable
}

async fn save_session(
    session: MqttSession,
    client_id: String,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let session_storage = SessionStorage::new(client_pool.clone());
    session_storage
        .set_session(client_id.clone(), &session)
        .await?;
    record_mqtt_session_created();
    Ok(())
}

async fn session_expiry_interval(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_properties: &Option<ConnectProperties>,
) -> u64 {
    let default_session_expiry_interval = cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .default_session_expiry_interval;
    let max_session_expiry_interval = cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .max_session_expiry_interval;

    let connection_session_expiry_interval = if let Some(properties) = connect_properties {
        if let Some(ck) = properties.session_expiry_interval {
            std::cmp::min(max_session_expiry_interval, ck)
        } else {
            default_session_expiry_interval
        }
    } else {
        default_session_expiry_interval
    };

    connection_session_expiry_interval as u64
}

#[cfg(test)]
mod test {
    use super::session_expiry_interval;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_config::broker::default_broker_config;
    use metadata_struct::mqtt::session::MqttSession;
    use protocol::mqtt::common::ConnectProperties;

    #[tokio::test]
    pub async fn build_session_test() {
        let client_id = "client_id_test-**".to_string();
        let session = MqttSession::new(client_id.clone(), 10, false, None, true);
        assert_eq!(client_id, session.client_id);
        assert_eq!(10, session.session_expiry_interval);
        assert!(!session.is_contain_last_will);
        assert!(session.last_will_delay_interval.is_none());

        assert!(session.connection_id.is_none());
        assert!(session.broker_id.is_none());
        assert!(session.reconnect_time.is_none());
        assert!(session.distinct_time.is_none());
    }

    #[tokio::test]
    pub async fn session_expiry_interval_test() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        cache_manager
            .broker_cache
            .set_cluster_config(default_broker_config())
            .await;
        let res = session_expiry_interval(&cache_manager, &None).await;
        assert_eq!(
            res,
            cache_manager
                .broker_cache
                .get_cluster_config()
                .await
                .mqtt_protocol_config
                .default_session_expiry_interval as u64
        );

        let properties = ConnectProperties {
            session_expiry_interval: Some(120),
            ..Default::default()
        };
        let res = session_expiry_interval(&cache_manager, &Some(properties)).await;
        assert_eq!(res, 120);

        let properties = ConnectProperties {
            session_expiry_interval: Some(3600),
            ..Default::default()
        };
        let res = session_expiry_interval(&cache_manager, &Some(properties)).await;
        assert_eq!(res, 1800);

        let properties = ConnectProperties {
            session_expiry_interval: None,
            ..Default::default()
        };
        let res = session_expiry_interval(&cache_manager, &Some(properties)).await;
        assert_eq!(res, 30);
    }
}
