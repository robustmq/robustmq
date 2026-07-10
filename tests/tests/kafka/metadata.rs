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
    use crate::kafka::common::consumer;
    use rdkafka::consumer::Consumer;
    use std::time::Duration;

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
    }
}
