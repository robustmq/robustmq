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
    use crate::mqtt::protocol::common::{
        broker_addr, build_conn_pros, build_create_conn_pros, create_test_env, distinct_conn,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use admin_server::cluster::ClusterConfigSetReq;
    use common_base::tools::unique_id;
    use common_config::config::BrokerConfig;
    use paho_mqtt::Client;
    use std::process;

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
        let create_opts = build_create_conn_pros(
            &client_test_properties.client_id,
            &client_test_properties.addr,
        );
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {err:?}");
            process::exit(1);
        });

        let conn_opts = build_conn_pros(client_test_properties.clone(), false);
        assert!(cli.connect(conn_opts.clone()).is_err());
    }

    fn test_correct_connect(client_test_properties: &ClientTestProperties) {
        let create_opts = build_create_conn_pros(
            &client_test_properties.client_id,
            &client_test_properties.addr,
        );
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {err:?}");
            process::exit(1);
        });

        let conn_opts = build_conn_pros(client_test_properties.clone(), false);

        assert!(cli.connect(conn_opts.clone()).is_ok());

        distinct_conn(cli);
    }

    async fn open_flapping_detect() {
        let admin_client = create_test_env().await;

        let mut config = BrokerConfig::default();
        config.mqtt_flapping_detect.enable = true;
        config.mqtt_flapping_detect.window_time = 60;
        config.mqtt_flapping_detect.max_client_connections = 20;
        config.mqtt_flapping_detect.ban_time = 1;

        let config_json = serde_json::to_string(&config).unwrap();
        let request = ClusterConfigSetReq {
            config_type: "broker".to_string(),
            config: config_json,
        };

        let reply = admin_client.set_cluster_config(&request).await;
        assert!(reply.is_ok());
    }

    async fn close_flapping_detect() {
        let admin_client = create_test_env().await;

        let mut config = BrokerConfig::default();
        config.mqtt_flapping_detect.enable = false;
        config.mqtt_flapping_detect.window_time = 60;
        config.mqtt_flapping_detect.max_client_connections = 20;
        config.mqtt_flapping_detect.ban_time = 1;

        let config_json = serde_json::to_string(&config).unwrap();
        let request = ClusterConfigSetReq {
            config_type: "broker".to_string(),
            config: config_json,
        };

        let reply = admin_client.set_cluster_config(&request).await;
        assert!(reply.is_ok());
    }
}
