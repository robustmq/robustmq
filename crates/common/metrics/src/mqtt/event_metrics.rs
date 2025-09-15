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

use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct ClientConnectionLabels {
    client_id: String,
}

common_base::register_counter_metric!(
    CLIENT_CONNECTION_COUNTER,
    "client_connections",
    "The number of client connections, regardless of success or failure.",
    ClientConnectionLabels
);

pub fn incr_client_connection_counter(client_id: String) {
    let labels = ClientConnectionLabels { client_id };
    common_base::counter_metric_inc!(CLIENT_CONNECTION_COUNTER, labels)
}

pub fn get_client_connection_counter(client_id: String) -> u64 {
    let labels = ClientConnectionLabels { client_id };
    let mut res = 0;
    common_base::counter_metric_get!(CLIENT_CONNECTION_COUNTER, labels, res);
    res
}

#[cfg(test)]
mod tests {
    use crate::mqtt::event_metrics;

    #[test]
    fn test_incr_client_connection_counter() {
        event_metrics::incr_client_connection_counter("test_client_1".to_string());

        assert_eq!(
            event_metrics::get_client_connection_counter("test_client_1".to_string()),
            1
        );

        event_metrics::incr_client_connection_counter("test_client_1".to_string());

        assert_eq!(
            event_metrics::get_client_connection_counter("test_client_1".to_string()),
            2
        );

        event_metrics::incr_client_connection_counter("test_client_2".to_string());

        assert_eq!(
            event_metrics::get_client_connection_counter("test_client_2".to_string()),
            1
        );
    }
}
