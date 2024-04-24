use crate::{
    metadata::{cluster::Cluster, session::Session},
    storage::session::SessionStorage,
};
use common_base::errors::RobustMQError;
use protocol::mqtt::{Connect, ConnectProperties};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

pub async fn save_connect_session<T>(
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
    let client_session = if connnect.clean_session {
        let session_storage = SessionStorage::new(storage_adapter.clone());
        match session_storage.get_session(&client_id).await {
            Ok(Some(session)) => {
                session.update_reconnect_time();
                session
            }
            Ok(None) => Session::build_session(
                &client_id,
                connnect,
                connect_properties,
                cluster.server_keep_alive(),
                contain_last_will,
            ),
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        Session::build_session(
            &client_id,
            connnect,
            connect_properties,
            cluster.server_keep_alive(),
            contain_last_will,
        )
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
