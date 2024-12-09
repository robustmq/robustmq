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

use common_base::tools::unique_id;
use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};

use crate::mqtt_protocol::common::{broker_addr, connect_server34, connect_server5, distinct_conn};

async fn publish34_qos(num: i32, qos: i32) {
    let mqtt_version = 3;
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server34(mqtt_version, &client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();
    for i in 0..num {
        let msg = Message::new(topic.clone(), format!("mqtt {i} message"), qos);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);

    let mqtt_version = 4;
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server34(mqtt_version, &client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();
    for i in 0..num {
        let msg = Message::new(topic.clone(), format!("mqtt {i} message"), qos);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);
}

async fn publish5_qos(num: i32, qos: i32, retained: bool) {
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server5(&client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();

    let mut props = Properties::new();
    props
        .push_u32(PropertyCode::MessageExpiryInterval, 50)
        .unwrap();
    for i in 0..num {
        let payload = format!("mqtt {i} message");
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(payload)
            .topic(topic.clone())
            .qos(qos)
            .retained(retained)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);
}

#[cfg(test)]
mod tests {

    use paho_mqtt::{QOS_0, QOS_1, QOS_2};

    use crate::mqtt_protocol::pub_qos_test::{publish34_qos, publish5_qos};

    #[tokio::test]
    async fn client34_publish_test() {
        let num = 10;
        publish34_qos(num, QOS_0).await;
        publish34_qos(num, QOS_1).await;
        publish34_qos(num, QOS_2).await;
    }

    #[tokio::test]
    async fn client5_publish_test() {
        let num = 1;
        publish5_qos(num, QOS_0, false).await;
        publish5_qos(num, QOS_1, false).await;
        publish5_qos(num, QOS_2, false).await;

        publish5_qos(num, QOS_0, true).await;
        publish5_qos(num, QOS_1, true).await;
        publish5_qos(num, QOS_2, true).await;
    }
}
