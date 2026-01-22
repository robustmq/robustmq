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
    use common_base::tools::unique_id;
    use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder, ReasonCode};

    #[tokio::test]
    // Scenario: MQTT v3 with client-provided (non-empty) client_id should connect successfully.
    async fn mqtt34_clean_session_true_test() {
        let addr = broker_addr_by_type("tcp");
        let client_id = unique_id();
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id.clone())
            .finalize();

        let cli = Client::new(create_opts).unwrap();
        let mut conn_opts = ConnectOptionsBuilder::with_mqtt_version(3);
        conn_opts
            .user_name(username())
            .password(password())
            .clean_session(true);
        let conn_opts = conn_opts.finalize();

        let result = cli.connect(conn_opts).unwrap();
        assert_eq!(result.reason_code(), ReasonCode::Success);
        distinct_conn(cli);
    }
}
