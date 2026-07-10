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
    use crate::kafka::common::{bootstrap_servers, consumer};
    use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
    use rdkafka::client::DefaultClientContext;
    use rdkafka::config::ClientConfig;
    use rdkafka::consumer::Consumer;
    use std::time::Duration;

    fn topic_exists(topic_name: &str) -> bool {
        let metadata = consumer()
            .fetch_metadata(None, Duration::from_secs(10))
            .expect("fetch cluster metadata");
        metadata.topics().iter().any(|t| t.name() == topic_name)
    }

    fn topic_partition_count(topic_name: &str) -> Option<usize> {
        let metadata = consumer()
            .fetch_metadata(None, Duration::from_secs(10))
            .expect("fetch cluster metadata");
        metadata
            .topics()
            .iter()
            .find(|t| t.name() == topic_name)
            .map(|t| t.partitions().len())
    }

    #[tokio::test]
    async fn create_topic_then_appears_in_list() {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers())
            .create()
            .expect("create kafka admin client");

        let topic_name = format!("it-topic-{}", uuid::Uuid::new_v4());

        assert!(
            !topic_exists(&topic_name),
            "topic unexpectedly present before creation"
        );

        let new_topic = NewTopic::new(&topic_name, 3, TopicReplication::Fixed(1));
        admin
            .create_topics([&new_topic], &AdminOptions::new())
            .await
            .expect("create topic request");

        assert_eq!(
            topic_partition_count(&topic_name),
            Some(3),
            "topic missing or wrong partition count after creation"
        );
    }
}
