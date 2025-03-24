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
    use crate::mqtt_protocol::common::{
        broker_addr, broker_grpc_addr, build_conn_pros, build_create_pros, distinct_conn,
    };
    use crate::mqtt_protocol::ClientTestProperties;
    use common_base::tools::unique_id;
    use grpc_clients::mqtt::admin::call::mqtt_broker_enable_flapping_detect;
    use grpc_clients::pool::ClientPool;
    use paho_mqtt::Client;
    use protocol::broker_mqtt::broker_mqtt_admin::EnableFlappingDetectRequest;
    use std::process;
    use std::sync::Arc;

    #[ignore = "reason"]
    #[tokio::test]
    async fn client_flapping_detect_test() {
        open_flapping_detect().await;

        let client_test_properties = ClientTestProperties {
            mqtt_version: 3,
            client_id: unique_id(),
            addr: broker_addr(),
            ws: false,
            ssl: false,
            ..Default::default()
        };

        for _i in 0..20 {
            test_correct_connect(&client_test_properties);
        }

        test_correct_connect(&client_test_properties);
        test_fail_connect(&client_test_properties);

        close_flapping_detect().await;
    }

    fn test_fail_connect(client_test_properties: &ClientTestProperties) {
        let create_opts = build_create_pros(
            &client_test_properties.client_id,
            &client_test_properties.addr,
        );
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_conn_pros(client_test_properties.clone(), false);
        assert!(cli.connect(conn_opts.clone()).is_err());
    }

    fn test_correct_connect(client_test_properties: &ClientTestProperties) {
        let create_opts = build_create_pros(
            &client_test_properties.client_id,
            &client_test_properties.addr,
        );
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_conn_pros(client_test_properties.clone(), false);

        assert!(cli.connect(conn_opts.clone()).is_ok());

        distinct_conn(cli);
    }

    async fn open_flapping_detect() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = EnableFlappingDetectRequest {
            is_enable: true,
            window_time: 60,
            max_client_connections: 20,
            ban_time: 1,
        };

        let _reply = mqtt_broker_enable_flapping_detect(&client_pool, &grpc_addr, request)
            .await
            .unwrap();
    }

    async fn close_flapping_detect() {
        let client_pool = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let request = EnableFlappingDetectRequest {
            is_enable: false,
            window_time: 60,
            max_client_connections: 20,
            ban_time: 1,
        };

        let _reply = mqtt_broker_enable_flapping_detect(&client_pool, &grpc_addr, request)
            .await
            .unwrap();
    }
}
