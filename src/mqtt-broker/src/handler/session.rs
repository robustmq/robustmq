use crate::{
    metadata::{cluster::Cluster, session::Session},
    storage::session::SessionStorage,
};
use common_base::errors::RobustMQError;
use protocol::mqtt::{Connect, ConnectProperties};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

pub async fn get_session_info<T>(
    client_id: String,
    contain_last_will: bool,
    cluster: &Cluster,
    connnect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    storage_adapter: Arc<T>,
) -> Result<Session, RobustMQError>
where
    T: StorageAdapter,
{
    let session_expiry = session_expiry_interval(cluster, connect_properties);
    let client_session = if connnect.clean_session {
        let session_storage = SessionStorage::new(storage_adapter.clone());
        match session_storage.get_session(&client_id).await {
            Ok(Some(mut session)) => {
                session.update_reconnect_time();
                session
            }
            Ok(None) => Session::new(client_id.clone(), session_expiry, contain_last_will),
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        Session::new(client_id.clone(), session_expiry, contain_last_will)
    };

    let session_storage = SessionStorage::new(storage_adapter);
    match session_storage
        .save_session(client_id, &client_session)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }
    return Ok(client_session);
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
