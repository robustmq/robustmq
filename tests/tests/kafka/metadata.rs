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
    use crate::kafka::common::{bootstrap_servers, consumer, set_auto_create_topics};
    use rdkafka::config::ClientConfig;
    use rdkafka::consumer::{BaseConsumer, Consumer};
    use std::time::Duration;

    fn topic_exists(topic_name: &str) -> bool {
        consumer()
            .fetch_metadata(None, Duration::from_secs(10))
            .expect("fetch cluster metadata")
            .topics()
            .iter()
            .any(|t| t.name() == topic_name)
    }

    #[tokio::test]
    async fn metadata_auto_creates_topic_when_enabled() {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers())
            .set("allow.auto.create.topics", "true")
            .create()
            .expect("create kafka consumer");

        let topic_name = format!("it-autocreate-{}", uuid::Uuid::new_v4());

        set_auto_create_topics(false).await;
        let _ = consumer.fetch_metadata(Some(&topic_name), Duration::from_secs(10));
        assert!(
            !topic_exists(&topic_name),
            "topic was created while auto-create was disabled"
        );

        set_auto_create_topics(true).await;

        let mut created = false;
        for _ in 0..10 {
            let _ = consumer.fetch_metadata(Some(&topic_name), Duration::from_secs(10));
            if topic_exists(&topic_name) {
                created = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        set_auto_create_topics(false).await;

        assert!(created, "topic was not auto-created after enabling the switch");
    }

    #[test]
    fn fetch_cluster_metadata_returns_brokers() {
        let consumer = consumer();
        let metadata = consumer
            .fetch_metadata(None, Duration::from_secs(10))
            .expect("fetch cluster metadata");

        println!("orig_broker_id: {}", metadata.orig_broker_id());
        println!("orig_broker_name: {}", metadata.orig_broker_name());
        println!("brokers ({}):", metadata.brokers().len());
        for broker in metadata.brokers() {
            println!(
                "  id={} host={} port={}",
                broker.id(),
                broker.host(),
                broker.port()
            );
        }
        println!("topics ({}):", metadata.topics().len());
        for topic in metadata.topics() {
            println!(
                "  name={} partitions={} error={:?}",
                topic.name(),
                topic.partitions().len(),
                topic.error()
            );
            for p in topic.partitions() {
                println!(
                    "    partition={} leader={} replicas={:?} isr={:?} error={:?}",
                    p.id(),
                    p.leader(),
                    p.replicas(),
                    p.isr(),
                    p.error()
                );
            }
        }

        assert!(
            !metadata.brokers().is_empty(),
            "cluster metadata returned no brokers"
        );
        for broker in metadata.brokers() {
            assert!(!broker.host().is_empty(), "broker host is empty");
            assert!(broker.port() > 0, "broker port is not set");
        }

        // rdkafka's Metadata omits controller_id; read it via the C API.
        let controller_id = unsafe {
            rdkafka::bindings::rd_kafka_controllerid(consumer.client().native_ptr(), 10_000)
        };
        println!("controller_id: {}", controller_id);
        assert!(
            controller_id >= 0,
            "no controller id reported by the cluster"
        );
        assert!(
            metadata.brokers().iter().any(|b| b.id() == controller_id),
            "controller id {} is not present in the broker list",
            controller_id
        );
    }
}
