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
    use paho_mqtt::{RetainHandling, SubscribeOptions, QOS_1};

    use crate::mqtt_protocol::common::{broker_addr, connect_server5, distinct_conn};

    #[tokio::test]
    async fn sub_exclusive_test() {
        let topic = format!("/tests/{}", unique_id());
        let exclusive_topic = format!("$exclusive{}", topic.clone());

        let addr = broker_addr();
        let sub_topics = &[topic.clone()];
        let sub_exclusive_topics: &[String; 1] = &[exclusive_topic.clone()];
        let sub_qos = &[QOS_1];
        let sub_opts = &[SubscribeOptions::new(
            true,
            false,
            RetainHandling::DontSendRetained,
        )];

        let client_id = unique_id();
        let cli = connect_server5(&client_id, &addr, false, false);

        // subscribe exclusive topic
        let consumer_client_id = unique_id();
        let consumer_cli = connect_server5(&consumer_client_id, &addr, false, false);
        let result =
            consumer_cli.subscribe_many_with_options(sub_exclusive_topics, sub_qos, sub_opts, None);
        assert!(result.is_ok());

        // subscribe topic success
        let consumer_client_id3 = unique_id();
        let consumer_cli3 = connect_server5(&consumer_client_id3, &addr, false, false);
        assert!(consumer_cli3
            .subscribe_many_with_options(sub_topics, sub_qos, sub_opts, None)
            .is_ok());

        // subscribe exclusive topic fail
        let consumer_client_id2 = unique_id();
        let consumer_cli2 = connect_server5(&consumer_client_id2, &addr, false, false);
        assert!(consumer_cli2
            .subscribe_many_with_options(sub_exclusive_topics, sub_qos, sub_opts, None)
            .is_err());

        distinct_conn(consumer_cli3);
        distinct_conn(consumer_cli2);
        distinct_conn(consumer_cli);
        distinct_conn(cli);
    }
}
