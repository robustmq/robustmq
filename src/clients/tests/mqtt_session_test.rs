mod common;
#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{
        placement::mqtt::call::{placement_list_session, placement_create_session,
            placement_update_session, placement_delete_session},
        poll::ClientPool,
    };
    use metadata_struct::mqtt::session::MQTTSession;
    use protocol::placement_center::generate::mqtt::{
        ListSessionRequest, CreateSessionRequest,
        UpdateSessionRequest, DeleteSessionRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_session_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = "test1".to_string();
        let connection_id: u64 = 1;
        let broker_id: u64 = 1;
        let reconnect_time: u64 = 10000000;
        let distinct_time: u64 = 10000000;

        let mqtt_session: MQTTSession = MQTTSession::default();


        let request = ListSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_list_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                for raw in data.sessions {
                    let session = serde_json::from_slice::<MQTTSession>(raw.as_slice()).unwrap();
                    println!("{:?}", session);
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = CreateSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
            session: MQTTSession::encode(&mqtt_session),
        };

        match placement_create_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = UpdateSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
            connection_id: connection_id.clone(),
            broker_id: broker_id.clone(),
            reconnect_time: reconnect_time.clone(),
            distinct_time: distinct_time.clone(),
        };

        match placement_update_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = DeleteSessionRequest {
            cluster_name: cluster_name.clone(),
            client_id: mqtt_session.client_id.clone(),
        };

        match placement_delete_session(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}