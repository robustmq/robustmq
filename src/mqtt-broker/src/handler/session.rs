use crate::{
    metadata::{cluster::Cluster, session::Session},
    storage::session::SessionStorage,
};
use common_base::errors::RobustMQError;
use protocol::mqtt::{Connect, ConnectProperties};
use std::sync::Arc;
use storage_adapter::memory::MemoryStorageAdapter;

pub async fn save_connect_session(
    auto_client_id: bool,
    client_id: String,
    contail_last_will: bool,
    cluster: Cluster,
    connnect: Connect,
    connect_properties: Option<ConnectProperties>,
    storage_adapter: Arc<MemoryStorageAdapter>,
) -> Result<Session, RobustMQError> {
    let mut client_session = Session::default();
    if !auto_client_id && connnect.clean_session {
        let session_storage = SessionStorage::new(storage_adapter.clone());
        match session_storage.get_session(&client_id).await {
            Ok(Some(da)) => {
                client_session = da;
            }
            Ok(None) => {
                client_session = client_session.build_session(
                    client_id.clone(),
                    connnect.clone(),
                    connect_properties.clone(),
                    cluster.server_keep_alive(),
                    contail_last_will,
                );
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    let session_storage = SessionStorage::new(storage_adapter.clone());
    match session_storage
        .save_session(client_id.clone(), client_session.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }
    return Ok(client_session);
}
