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
    use std::time::Duration;

    use admin_server::mqtt::connector::{
        ConnectorDetailReq, ConnectorDetailResp, ConnectorListReq, ConnectorListRow,
        CreateConnectorReq,
    };
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::bridge::config_local_file::{
        LocalFileConnectorConfig, RotationStrategy,
    };
    use paho_mqtt::MessageBuilder;
    use tokio::time::sleep;

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, ssl_by_type, ws_by_type,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn file_connector_test() {
        let admin_client = create_test_env().await;
        let topic = "/test/t1";
        let connector_name = unique_id();
        let config = LocalFileConnectorConfig {
            local_file_path: format!("/tmp/{}.log", unique_id()),
            max_size_gb: 1,
            rotation_strategy: RotationStrategy::Size,
        };
        let connector = CreateConnectorReq {
            connector_name: connector_name.clone(),
            connector_type: "file".to_string(),
            topic_name: topic.to_string(),
            config: serde_json::to_string(&config).unwrap(),
        };

        let res = admin_client.create_connector(&connector).await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(10)).await;

        let request = ConnectorListReq {
            connector_name: Some(connector_name.clone()),
            ..Default::default()
        };
        let results = admin_client
            .get_connector_list::<ConnectorListReq, Vec<ConnectorListRow>>(&request)
            .await
            .unwrap();
        assert_eq!(results.data.len(), 1);

        let connector = results.data.first().unwrap();
        assert_eq!(connector.status, "Running".to_string());

        // send message
        let network = "tcp";
        let qos = 1;
        let client_id = build_client_id(format!("file_connector_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // publish retain
        let message = "file_connector_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic)
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, false);
        distinct_conn(cli);

        sleep(Duration::from_secs(5)).await;
        // get status
        let request = ConnectorDetailReq {
            connector_name: connector_name.clone(),
        };
        let results = admin_client
            .get_connector_detail::<ConnectorDetailReq, ConnectorDetailResp>(&request)
            .await
            .unwrap();
        assert_eq!(results.send_success_total, 1);
    }
}
