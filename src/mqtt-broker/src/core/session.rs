use std::sync::Arc;

use crate::{
    metadata::{
        cluster::Cluster,
        session::{LastWillData, Session},
    },
    storage::session::SessionStorage,
};
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use protocol::mqtt::{Connect, ConnectProperties, LastWill, LastWillProperties};

pub async fn build_session(
    connect_id: u64,
    client_id: String,
    cluster: Cluster,
    connnect: Connect,
    connect_properties: Option<ConnectProperties>,
    last_will: Option<LastWill>,
    last_will_properties: Option<LastWillProperties>,
    client_poll: Arc<ClientPool>,
) -> Result<(Session, bool), RobustMQError> {
    let session_expiry = session_expiry_interval(cluster, connect_properties);

    let delay_interval = if let Some(properties) = last_will_properties.clone() {
        if let Some(value) = properties.delay_interval {
            value
        } else {
            0
        }
    } else {
        0
    };

    let last_will = if last_will.is_none() {
        None
    } else {
        Some(LastWillData {
            last_will,
            last_will_properties,
        })
    };

    let (mut session, new_session) = if connnect.clean_session {
        let session_storage = SessionStorage::new(client_poll.clone());
        match session_storage.get_session(&client_id).await {
            Ok(Some(mut session)) => {
                session.update_reconnect_time();
                (session, false)
            }
            Ok(None) => (
                Session::new(client_id.clone(), session_expiry, last_will, delay_interval),
                true,
            ),
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        (
            Session::new(client_id.clone(), session_expiry, last_will, delay_interval),
            true,
        )
    };

    session.update_connnction_id(connect_id);
    if !new_session {
        session.update_reconnect_time();
    }

    let session_storage = SessionStorage::new(client_poll);
    match session_storage.save_session(client_id, &session).await {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }
    return Ok((session, new_session));
}

fn session_expiry_interval(cluster: Cluster, connect_properties: Option<ConnectProperties>) -> u32 {
    let session_expiry_interval = if let Some(properties) = connect_properties {
        if let Some(ck) = properties.session_expiry_interval {
            ck
        } else {
            u32::MAX
        }
    } else {
        u32::MAX
    };
    return std::cmp::min(cluster.session_expiry_interval, session_expiry_interval);
}
