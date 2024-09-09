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


use super::{cache::CacheManager, lastwill::last_will_delay_interval};
use crate::storage::session::SessionStorage;
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, error::common::CommonError, tools::now_second,
};
use metadata_struct::mqtt::session::MQTTSession;
use protocol::mqtt::common::{Connect, ConnectProperties, LastWill, LastWillProperties};
use std::sync::Arc;

pub async fn build_session(
    connect_id: u64,
    client_id: &String,
    connnect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    client_poll: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
) -> Result<(MQTTSession, bool), CommonError> {
    let session_expiry = session_expiry_interval(cache_manager, connect_properties);
    let is_contain_last_will = !last_will.is_none();
    let last_will_delay_interval = last_will_delay_interval(&last_will_properties);

    let (mut session, new_session) = if connnect.clean_session {
        let session_storage = SessionStorage::new(client_poll.clone());
        match session_storage.get_session(client_id.clone()).await {
            Ok(Some(session)) => (session, false),
            Ok(None) => (
                MQTTSession::new(
                    &client_id,
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
    } else {
        (
            MQTTSession::new(
                &client_id,
                session_expiry,
                is_contain_last_will,
                last_will_delay_interval,
            ),
            true,
        )
    };

    let conf = broker_mqtt_conf();
    session.update_connnction_id(Some(connect_id));
    session.update_broker_id(Some(conf.broker_id));
    session.update_reconnect_time();
    return Ok((session, new_session));
}

pub async fn save_session(
    connect_id: u64,
    session: MQTTSession,
    new_session: bool,
    client_id: &String,
    client_poll: &Arc<ClientPool>,
) -> Result<(), CommonError> {
    let conf = broker_mqtt_conf();
    let session_storage = SessionStorage::new(client_poll.clone());
    if new_session {
        match session_storage.set_session(client_id, &session).await {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        match session_storage
            .update_session(&client_id, connect_id, conf.broker_id, now_second(), 0)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }

    return Ok(());
}

fn session_expiry_interval(
    cache_manager: &Arc<CacheManager>,
    connect_properties: &Option<ConnectProperties>,
) -> u64 {
    let cluster_session_expiry_interval = cache_manager.get_cluster_info().protocol.session_expiry_interval;
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

#[cfg(test)]
mod test {
    use super::session_expiry_interval;
    use crate::handler::cache::CacheManager;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use metadata_struct::mqtt::session::MQTTSession;
    use protocol::mqtt::common::ConnectProperties;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn build_session_test() {
        let client_id = "client_id_test-**".to_string();
        let session = MQTTSession::new(&client_id, 10, false, None);
        assert_eq!(client_id, session.client_id);
        assert_eq!(10, session.session_expiry);
        assert!(!session.is_contain_last_will);
        assert!(session.last_will_delay_interval.is_none());
        
        assert!(session.connection_id.is_none());
        assert!(session.broker_id.is_none());
        assert!(session.reconnect_time.is_none());
        assert!(session.distinct_time.is_none());
    }

    #[test]
    pub fn session_expiry_interval_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();
        let client_poll = Arc::new(ClientPool::new(100));
        let cache_manager = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));
        let res = session_expiry_interval(&cache_manager, &None);
        assert_eq!(
            res,
            cache_manager.get_cluster_info().protocol.session_expiry_interval as u64
        );

        let mut properteis = ConnectProperties::default();
        properteis.session_expiry_interval = Some(120);
        let res = session_expiry_interval(&cache_manager, &Some(properteis));
        assert_eq!(res, 120);

        let mut properteis = ConnectProperties::default();
        properteis.session_expiry_interval = Some(3600);
        let res = session_expiry_interval(&cache_manager, &Some(properteis));
        assert_eq!(res, 1800);

        let mut properteis = ConnectProperties::default();
        properteis.session_expiry_interval = None;
        let res = session_expiry_interval(&cache_manager, &Some(properteis));
        assert_eq!(res, 1800);
    }
}
