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

use crate::observability::metrics::load_global_registry;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use std::sync::LazyLock;

/**
For metrics related to counters,
we typically name them in the format *xxx_counter* to indicate that they are of the counter type.
If its type is *Family<>*, then we usually define labels as *xxxLabels*.
For example, for *client_connection_counter*, its labels would be *ClientConnectionLabels*.
*/
static EVENT_METRICS: LazyLock<EventMetrics> = LazyLock::new(EventMetrics::init);

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct ClientConnectionLabels {
    client_id: String,
}

struct EventMetrics {
    client_connection_counter: Family<ClientConnectionLabels, Counter>,
}

impl EventMetrics {
    fn init() -> Self {
        // init event_metrics
        let event_metrics = Self {
            client_connection_counter: Family::default(),
        };

        // get global registry
        let mut global_registry = load_global_registry();

        // register event_metrics
        global_registry.register(
            "client_connections",
            "The number of client connections, regardless of success or failure.",
            event_metrics.client_connection_counter.clone(),
        );

        // return event_metrics
        event_metrics
    }
}

pub fn incr_client_connection_counter(client_id: String) {
    let labels = ClientConnectionLabels { client_id };

    EVENT_METRICS
        .client_connection_counter
        .get_or_create(&labels)
        .inc();
}

pub fn get_client_connection_counter(client_id: String) -> u64 {
    let labels = ClientConnectionLabels { client_id };

    EVENT_METRICS
        .client_connection_counter
        .get_or_create(&labels)
        .get()
}

#[cfg(test)]
mod tests {
    use crate::observability::metrics::event_metrics;

    #[test]
    fn test_incr_client_connection_counter() {
        event_metrics::incr_client_connection_counter("test_client_1".to_string());

        assert_eq!(
            event_metrics::get_client_connection_counter("test_client_1".to_string()),
            1
        );
    }
}
