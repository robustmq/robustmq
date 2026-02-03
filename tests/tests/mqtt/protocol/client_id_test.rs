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
    use crate::mqtt::protocol::common::{broker_addr_by_type, distinct_conn, password, username};
    use common_base::uuid::unique_id;
    use paho_mqtt::{
        Client, ConnectOptionsBuilder, CreateOptionsBuilder, Properties, PropertyCode, ReasonCode,
    };

    #[tokio::test]
    // Scenario: MQTT v3 with client-provided (non-empty) client_id should connect successfully.
    async fn mqtt3_client_id_test() {
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
    }

    #[tokio::test]
    // Scenario: MQTT v3 with empty client_id must be rejected (connect should fail).
    async fn mqtt3_empty_client_id_test() {
        let addr = broker_addr_by_type("tcp");
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id("")
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
        conn_opts.user_name(username()).password(password());
        let conn_opts = conn_opts.finalize();

        assert!(cli.connect(conn_opts).is_err());
    }

    #[tokio::test]
    // Scenario: MQTT v4 (3.1.1) with client-provided (non-empty) client_id should connect successfully.
    async fn mqtt4_client_id_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id)
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(4);
        conn_opts.user_name(username()).password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);
        distinct_conn(cli);
    }

    #[tokio::test]
    // Scenario: MQTT v4 (3.1.1) with empty client_id and clean_session=true should connect successfully.
    async fn mqtt4_empty_client_id_clean_session_true_test() {
        let addr = broker_addr_by_type("tcp");
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id("")
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(4);
        conn_opts
            .clean_session(true)
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);
        distinct_conn(cli);
    }

    #[tokio::test]
    // Scenario: MQTT v4 (3.1.1) with empty client_id and clean_session=false must be rejected (connect should fail).
    async fn mqtt4_empty_client_id_clean_session_false_test() {
        let addr = broker_addr_by_type("tcp");
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id("")
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(4);
        conn_opts
            .clean_session(false)
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();

        assert!(cli.connect(conn_opts).is_err());
    }

    #[tokio::test]
    // Scenario: MQTT v5 with client-provided (non-empty) client_id should NOT receive AssignedClientIdentifer.
    async fn mqtt5_client_id_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id.clone())
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let props = Properties::new();
        let mut conn_opts = ConnectOptionsBuilder::new_v5();
        conn_opts
            .properties(props.clone())
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);

        let resp_pros = result.properties();
        assert!(resp_pros
            .get(PropertyCode::AssignedClientIdentifer)
            .is_none());
        distinct_conn(cli);
    }

    #[tokio::test]
    // Scenario: MQTT v5 with empty client_id should be auto-assigned and returned via AssignedClientIdentifer.
    async fn mqtt5_auto_assigned_client_id_test() {
        let addr = broker_addr_by_type("tcp");
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id("")
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let props = Properties::new();
        let mut conn_opts = ConnectOptionsBuilder::new_v5();
        conn_opts
            .properties(props.clone())
            .user_name(username())
            .password(password());
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);

        let resp_pros = result.properties();
        let assign_client_id = resp_pros
            .get(PropertyCode::AssignedClientIdentifer)
            .unwrap()
            .get_string()
            .unwrap();
        assert!(!assign_client_id.is_empty());
        assert_eq!(assign_client_id.len(), unique_id().len());

        distinct_conn(cli);
    }
}
