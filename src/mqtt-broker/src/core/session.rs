use crate::{
    metadata::{cluster::Cluster, session::Session},
    storage::session::SessionStorage,
};
use common_base::errors::RobustMQError;
use protocol::mqtt::{Connect, ConnectProperties, LastWillProperties};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

pub async fn get_session_info<T>(
    connect_id: u64,
    client_id: String,
    contain_last_will: bool,
    last_will_properties: Option<LastWillProperties>,
    cluster: &Cluster,
    connnect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    storage_adapter: Arc<T>,
) -> Result<(Session, bool), RobustMQError>
where
    T: StorageAdapter,
{
    let session_expiry = session_expiry_interval(cluster, connect_properties);
    let delay_interval = if let Some(properties) = last_will_properties {
        if let Some(value) = properties.delay_interval {
            value
        } else {
            0
        }
    } else {
        0
    };
    let (mut session, new_session) = if connnect.clean_session {
        let session_storage = SessionStorage::new(storage_adapter.clone());
        match session_storage.get_session(&client_id).await {
            Ok(Some(mut session)) => {
                session.update_reconnect_time();
                (session, false)
            }
            Ok(None) => (
                Session::new(
                    client_id.clone(),
                    session_expiry,
                    contain_last_will,
                    delay_interval,
                ),
                true,
            ),
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        (
            Session::new(
                client_id.clone(),
                session_expiry,
                contain_last_will,
                delay_interval,
            ),
            true,
        )
    };

    session.update_connnction_id(connect_id);
    if !new_session {
        session.update_reconnect_time();
    }

    let session_storage = SessionStorage::new(storage_adapter);
    match session_storage.save_session(client_id, &session).await {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }
    return Ok((session, new_session));
}

fn session_expiry_interval(
    cluster: &Cluster,
    connect_properties: &Option<ConnectProperties>,
) -> u32 {
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
