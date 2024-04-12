use crate::{metadata::session::Session, storage::session::SessionStorage};

pub fn connect_session(auto_client_id: bool,clean_session:bool) {
    let mut client_session = Session::default();
    if !auto_client_id && clean_session {
        let session_storage = SessionStorage::new(self.storage_adapter.clone());
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
                    !last_will.is_none(),
                );
            }
            Err(_) => {
                return self.ack_build.distinct(DisconnectReasonCode::NotAuthorized);
            }
        }
    }

    let session_storage = SessionStorage::new(self.storage_adapter.clone());
    match session_storage
        .save_session(client_id.clone(), client_session.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error(e.to_string());
            return self
                .ack_build
                .distinct(DisconnectReasonCode::UnspecifiedError);
        }
    }
}
