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
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode, QOS_1};

    use crate::mqtt_client::mqtt::common::{
        broker_addr, connect_server5, connect_server5_response_information, distinct_conn,
    };

    #[tokio::test]
    async fn client5_reuqset_response_test() {
        let sub_qos = &[0, 0];

        let topic = unique_id();
        let request_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        simple_test(
            request_topic,
            response_topic,
            sub_qos,
            "2".to_string(),
            None,
            false,
        )
        .await;

        // correlation data
        let topic = unique_id();
        let request_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        let data = "123456".to_string();
        simple_test(
            request_topic,
            response_topic,
            sub_qos,
            "2".to_string(),
            Some(data),
            false,
        )
        .await;

        // connect response information
        let topic = unique_id();
        let request_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        simple_test(
            request_topic,
            response_topic,
            sub_qos,
            "2".to_string(),
            None,
            true,
        )
        .await;

        // connect response information correlation data
        let topic = unique_id();
        let request_topic = format!("/request/{}", topic);
        let response_topic = format!("/response/{}", topic);
        let data = "123456".to_string();
        simple_test(
            request_topic,
            response_topic,
            sub_qos,
            "2".to_string(),
            Some(data),
            true,
        )
        .await;
    }

    async fn simple_test(
        request_topic: String,
        response_topic: String,
        sub_qos: &[i32],
        payload_flag: String,
        correlation_data: Option<String>,
        connect_response_information: bool,
    ) {
        let client_id = unique_id();
        let addr = broker_addr();
        let sub_topics = &[request_topic.clone(), response_topic.clone()];

        let (cli, response_topic) = match connect_response_information {
            true => {
                let (cli, response_information) =
                    connect_server5_response_information(&client_id, &addr);
                (
                    cli,
                    format!("{response_information}{}", &response_topic[1..]),
                )
            }
            false => (
                connect_server5(&client_id, &addr, false, false),
                response_topic,
            ),
        };

        let message_content = format!("mqtt {payload_flag} message");

        // publish
        let mut props: Properties = Properties::new();
        props
            .push_string(PropertyCode::ResponseTopic, response_topic.as_str())
            .unwrap();
        if let Some(correlation_data) = correlation_data.as_ref() {
            props
                .push_val(PropertyCode::CorrelationData, correlation_data.clone())
                .unwrap();
        }
        let msg = MessageBuilder::new()
            .properties(props)
            .topic(request_topic.clone())
            .payload(message_content.clone())
            .qos(QOS_1)
            .finalize();

        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        // subscribe
        let rx = cli.start_consuming();
        match cli.subscribe(request_topic.as_str(), sub_qos[0]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);

            let topic = msg
                .properties()
                .get_string(PropertyCode::ResponseTopic)
                .unwrap();
            assert_eq!(topic, response_topic);

            println!("{msg:?}");
            println!("{topic:?}");

            let msg = match msg.properties().get_binary(PropertyCode::CorrelationData) {
                Some(data) => {
                    let mut props: Properties = Properties::new();
                    props.push_val(PropertyCode::CorrelationData, data).unwrap();
                    MessageBuilder::new()
                        .topic(topic)
                        .payload(payload.clone())
                        .qos(QOS_1)
                        .properties(props)
                        .finalize()
                }
                _ => Message::new(topic, payload.clone(), QOS_1),
            };
            cli.publish(msg).unwrap();
        }

        // subscribe
        let rx = cli.start_consuming();
        match cli.subscribe(response_topic.as_str(), sub_qos[1]) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        if let Some(msg) = rx.iter().next() {
            let msg = msg.unwrap();
            println!("{msg:?}");

            if connect_response_information {
                assert!(!msg.topic().starts_with(&sub_topics[1]))
            }

            let payload = String::from_utf8(msg.payload().to_vec()).unwrap();
            assert_eq!(payload, message_content);
            assert_eq!(
                correlation_data.map(|v| v.as_bytes().to_vec()),
                msg.properties().get_binary(PropertyCode::CorrelationData)
            );
        }
        distinct_conn(cli);
    }
}
