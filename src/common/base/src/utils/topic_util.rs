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

use regex::Regex;

pub fn topic_name_regex_match(topic_name1: &str, topic_name2: &str) -> bool {
    base_topic_name_regex_match(topic_name1, topic_name2)
        || base_topic_name_regex_match(topic_name2, topic_name1)
}

pub fn base_topic_name_regex_match(topic_name: &str, regex_topic_name: &str) -> bool {
    // Topic name perfect matching
    if topic_name == regex_topic_name {
        return true;
    }

    if regex_topic_name.contains("+") {
        let regex_str = regex_topic_name.replace("+", "[^+*/]+");
        let regex = Regex::new(&regex_str).unwrap();
        return regex.is_match(topic_name);
    }

    if regex_topic_name.contains("#") {
        if regex_topic_name.split("/").last().unwrap() != "#" {
            return false;
        }
        let regex_str = regex_topic_name.replace("#", "[^+#]+");
        let regex = Regex::new(&regex_str).unwrap();
        return regex.is_match(topic_name);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_name_regex_match() {
        assert!(topic_name_regex_match("topic", "topic"));
        assert!(topic_name_regex_match("/topic/sub", "/topic/+"));
        assert!(topic_name_regex_match("/topic/+/topic", "/topic/sub/topic"));
        assert!(topic_name_regex_match("/topic/#", "/topic/sub"));
        assert!(topic_name_regex_match("/topic/sub/topic", "/topic/#"));
        assert!(!topic_name_regex_match("/topic/sub/topic", "/topic/+/sub"));
        assert!(!topic_name_regex_match("topic/sub/topic", "another/#"));
    }

    #[test]
    fn test_base_topic_name_regex_match() {
        assert!(base_topic_name_regex_match("topic", "topic"));
        assert!(base_topic_name_regex_match("/topic/sub", "/topic/+"));
        assert!(base_topic_name_regex_match(
            "/topic/sub/topic",
            "/topic/+/topic"
        ));
        assert!(base_topic_name_regex_match("/topic/sub", "/topic/#"));
        assert!(base_topic_name_regex_match("/topic/sub/topic", "/topic/#"));
        assert!(!base_topic_name_regex_match(
            "/topic/sub/topic",
            "/topic/+/sub"
        ));
        assert!(!base_topic_name_regex_match("topic/sub/topic", "another/#"));
    }
}
