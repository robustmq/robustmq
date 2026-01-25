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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn, ssl_by_type,
            ws_by_type,
        },
        ClientTestProperties,
    };
    use common_base::tools::unique_id;
    use paho_mqtt::{MessageBuilder, QOS_1};

    #[tokio::test]
    async fn problem_info_test() {
        let network = "tcp";
        let topic = format!("/problem_info_test/{}/{}", unique_id(), network);

        let client_id = build_client_id(format!("problem_info_test_{network}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            packet_size: Some(128),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let packet_size: usize = 128;

        // len = packet_size + 1
        let msg = MessageBuilder::new()
            .topic(topic.to_owned())
            .payload("a".repeat(packet_size + 1))
            .qos(QOS_1)
            .finalize();

        if let Err(e) = cli.publish(msg) {
            println!("{}", e.to_string());
        }

        distinct_conn(cli);
    }
}
