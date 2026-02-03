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

#[cfg(test)]
mod tests {
    use admin_server::{
        mqtt::client::{ClientListReq, ClientListRow},
        tool::PageReplyData,
    };
    use common_base::uuid::unique_id;
    use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder, ReasonCode};
    use protocol::robust::RobustMQProtocol;

    use crate::mqtt::protocol::common::{broker_addr_by_type, create_test_env, password, username};

    #[tokio::test]
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
        let request = ClientListReq {
            client_id: Some(client_id.clone()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<ClientListRow>> =
            admin_client.get_client_list(&request).await.unwrap();
        let mut flag = false;
        for raw in data.data {
            let nc = raw.network_connection.unwrap();
            if raw.client_id == client_id && nc.protocol.unwrap() == RobustMQProtocol::MQTT3 {
                flag = true
            }
        }
        assert!(flag);
    }

    #[tokio::test]
    async fn mqtt4_protocol_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id.clone())
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(4);
        conn_opts.user_name(username()).password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);

        let admin_client = create_test_env().await;
        let request = ClientListReq {
            client_id: Some(client_id.clone()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<ClientListRow>> =
            admin_client.get_client_list(&request).await.unwrap();
        let mut flag = false;
        for raw in data.data {
            let nc = raw.network_connection.unwrap();
            if raw.client_id == client_id && nc.protocol.unwrap() == RobustMQProtocol::MQTT4 {
                flag = true
            }
        }
        assert!(flag);
    }

    #[tokio::test]
    async fn mqtt5_protocol_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id.clone())
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::new_v5();
        conn_opts.user_name(username()).password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);

        let admin_client = create_test_env().await;
        let request = ClientListReq {
            client_id: Some(client_id.clone()),
            ..Default::default()
        };
        let data: PageReplyData<Vec<ClientListRow>> =
            admin_client.get_client_list(&request).await.unwrap();
        let mut flag = false;
        for raw in data.data {
            let nc = raw.network_connection.unwrap();
            if raw.client_id == client_id && nc.protocol.unwrap() == RobustMQProtocol::MQTT5 {
                flag = true
            }
        }
        assert!(flag);
    }
}
