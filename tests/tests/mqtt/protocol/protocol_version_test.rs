#[cfg(test)]
mod tests {
    use admin_server::{
        mqtt::{
            client::ClientListReq,
            session::{SessionListReq, SessionListRow},
        },
        tool::PageReplyData,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder, ReasonCode};

    use crate::mqtt::protocol::common::{broker_addr_by_type, create_test_env, password, username};

    #[tokio::test]
    // Scenario: MQTT v3 with client-provided (non-empty) client_id should connect successfully.
    async fn mqtt3_protocol_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id.clone())
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
        conn_opts.user_name(username()).password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);

        let admin_client = create_test_env().await;
        let request = SessionListReq {
            client_id: Some(client_id),
            ..Default::default()
        };
        let data: PageReplyData<SessionListRow> =
            admin_client.get_session_list(&request).await.unwrap();
        println!("{:?}", data);
    }
}
