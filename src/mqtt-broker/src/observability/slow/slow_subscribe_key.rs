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

use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SlowSubscribeKey {
    pub time_span: u64,
    pub client_id: String,
    pub topic_name: String,
}

impl SlowSubscribeKey {
    pub fn build(time_span: u64, client_id: String, topic_name: String) -> Self {
        Self {
            time_span,
            client_id,
            topic_name,
        }
    }

    pub fn time_span(&self) -> u64 {
        self.time_span
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }
}

impl PartialOrd for SlowSubscribeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SlowSubscribeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_span
            .cmp(&other.time_span)
            .reverse()
            .then_with(|| self.client_id.cmp(&other.client_id))
            .then_with(|| self.topic_name.cmp(&other.topic_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_different_time_span() {
        let keys = vec![
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 5,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeKey {
                time_span: 15,
                client_id: "client3".into(),
                topic_name: "topicC".into(),
            },
        ];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(
            sorted_keys,
            vec![
                SlowSubscribeKey {
                    time_span: 15,
                    client_id: "client3".into(),
                    topic_name: "topicC".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
                SlowSubscribeKey {
                    time_span: 5,
                    client_id: "client2".into(),
                    topic_name: "topicB".into()
                },
            ]
        );
    }

    #[test]
    fn test_same_time_span_different_client_id() {
        let keys = vec![
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client3".into(),
                topic_name: "topicC".into(),
            },
        ];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(
            sorted_keys,
            vec![
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client2".into(),
                    topic_name: "topicB".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client3".into(),
                    topic_name: "topicC".into()
                },
            ]
        );
    }

    #[test]
    fn test_same_time_span_and_client_id_different_topic_name() {
        let keys = vec![
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicC".into(),
            },
        ];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(
            sorted_keys,
            vec![
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicB".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicC".into()
                },
            ]
        );
    }

    #[test]
    fn test_duplicate_keys() {
        let keys = vec![
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
        ];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(
            sorted_keys,
            vec![
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
                SlowSubscribeKey {
                    time_span: 10,
                    client_id: "client1".into(),
                    topic_name: "topicA".into()
                },
            ]
        );
    }

    #[test]
    fn test_single_element() {
        let keys = vec![SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        }];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(
            sorted_keys,
            vec![SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into()
            },]
        );
    }

    #[test]
    fn test_empty_collection() {
        let keys: Vec<SlowSubscribeKey> = vec![];

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();

        assert_eq!(sorted_keys, vec![]);
    }
}
