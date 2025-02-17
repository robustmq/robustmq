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

#[derive(Debug, PartialEq)]
enum SubscriptionType {
    SharedWithGroup {
        function_prefix: String,
        group_name: String,
    },
    SharedWithoutGroup {
        function_prefix: String,
    },
}

#[derive(Debug, PartialEq)]
struct SharedSubscription {
    subscription_type: SubscriptionType,
    topic_pattern: String,
}

impl SharedSubscription {
    pub(crate) fn new(topic_name_with_function_prefix: &str) -> Self {
        let mut split_topic_name_with_function_prefix = topic_name_with_function_prefix
            .split('/')
            .collect::<Vec<&str>>();

        let subscription_type =
            Self::get_subscription_type(&mut split_topic_name_with_function_prefix);

        let topic_pattern = Self::get_topic_pattern(&mut split_topic_name_with_function_prefix);

        SharedSubscription {
            subscription_type,
            topic_pattern,
        }
    }

    fn get_subscription_type(
        mut split_topic_name_with_function_prefix: &mut Vec<&str>,
    ) -> SubscriptionType {
        let function_prefix = split_topic_name_with_function_prefix.remove(0);
        println!("{}", function_prefix);
        match function_prefix {
            "$share" => SubscriptionType::SharedWithGroup {
                function_prefix: function_prefix.to_string(),
                group_name: Self::get_group_name(&mut split_topic_name_with_function_prefix),
            },
            "$queue" => SubscriptionType::SharedWithoutGroup {
                function_prefix: function_prefix.to_string(),
            },
            _ => {
                panic!("Invalid function prefix")
            }
        }
    }

    fn get_topic_pattern(split_topic_name_with_function_prefix: &mut Vec<&str>) -> String {
        split_topic_name_with_function_prefix.join("/")
    }

    fn get_group_name(split_topic_name_with_function_prefix: &mut Vec<&str>) -> String {
        split_topic_name_with_function_prefix.remove(0).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::matchers::{anything, eq, matches_pattern};
    use googletest::{assert_that, gtest};

    // $share/group1/topic -> SharedSubscription { subscription_type: Shared { group_name: "group1".to_string() }, topic_pattern: "topic".to_string() }
    #[gtest]
    fn should_return_shared_subscription_is_anything() {
        assert_that!(
            SharedSubscription::new("$share/group1/topic"),
            matches_pattern!(SharedSubscription {
                subscription_type: anything(),
                topic_pattern: anything()
            })
        )
    }

    #[gtest]
    #[should_panic]
    fn should_panic_shared_subscription_invalid_function_prefix() {
        SharedSubscription::new("$invalid/group1/topic");
    }

    #[gtest]
    fn should_return_subscription_type() {
        let subscription = SharedSubscription::new("$share/group1/topic");

        assert_that!(
            subscription.subscription_type,
            matches_pattern!(SubscriptionType::SharedWithGroup {
                function_prefix: eq("$share"),
                group_name: anything()
            })
        );
    }

    #[gtest]
    fn should_return_subscription_type_is_not_empty() {
        let subscription = SharedSubscription::new("$share/group1/topic");

        assert_that!(
            subscription.subscription_type,
            matches_pattern!(SubscriptionType::SharedWithGroup {
                function_prefix: eq("$share"),
                group_name: eq("group1")
            })
        )
    }

    #[gtest]
    fn should_return_subscription_type_is_shared_without_group() {
        let subscription = SharedSubscription::new("$queue/topic");

        assert_that!(
            subscription.subscription_type,
            matches_pattern!(SubscriptionType::SharedWithoutGroup {
                function_prefix: eq("$queue")
            })
        )
    }

    #[gtest]
    fn should_return_topic_pattern_1() {
        let subscription = SharedSubscription::new("$share/group1/topic");

        assert_that!(subscription.topic_pattern, eq("topic"))
    }

    #[gtest]
    fn should_return_topic_pattern_2() {
        let subscription = SharedSubscription::new("$share/group1/topic/1");

        assert_that!(subscription.topic_pattern, eq("topic/1"))
    }

    #[gtest]
    fn should_return_topic_pattern_3() {
        let subscription = SharedSubscription::new("$share/group1/topic/1/#");

        assert_that!(subscription.topic_pattern, eq("topic/1/#"))
    }

    #[gtest]
    fn should_return_topic_pattern_4() {
        let subscription = SharedSubscription::new("$share/group1/topic/1/+/2");

        assert_that!(subscription.topic_pattern, eq("topic/1/+/2"))
    }

    #[gtest]
    fn should_return_shared_subscription() {
        let subscription = SharedSubscription::new("$share/group1/topic");

        assert_that!(
            subscription,
            eq(&SharedSubscription {
                subscription_type: SubscriptionType::SharedWithGroup {
                    function_prefix: "$share".to_string(),
                    group_name: "group1".to_string()
                },
                topic_pattern: "topic".to_string()
            })
        )
    }

    #[gtest]
    fn should_return_shared_subscription_2() {
        let subscription = SharedSubscription::new("$queue/topic");

        assert_that!(
            subscription,
            eq(&SharedSubscription {
                subscription_type: SubscriptionType::SharedWithoutGroup {
                    function_prefix: "$queue".to_string()
                },
                topic_pattern: "topic".to_string()
            })
        )
    }
}
