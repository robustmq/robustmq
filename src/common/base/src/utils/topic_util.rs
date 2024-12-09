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
const EXCLUSIVE_SUB_PREFIX: &str = "$exclusive";
pub fn is_exclusive_sub(sub_path: String) -> bool {
    sub_path.starts_with(EXCLUSIVE_SUB_PREFIX)
}
pub fn decode_exclusive_sub_path_to_topic_name(sub_path: String) -> String {
    let mut str_slice: Vec<&str> = sub_path.split("/").collect();
    str_slice.remove(0);
    format!("/{}", str_slice.join("/"))
}

pub fn exclusive_sub_path_regex_match(sub_path1: String, sub_path2: String) -> bool {
    let sub_topic_name1 = if is_exclusive_sub(sub_path1.clone()) {
        decode_exclusive_sub_path_to_topic_name(sub_path1)
    } else {
        sub_path1
    };

    let sub_topic_name2 = if is_exclusive_sub(sub_path2.clone()) {
        decode_exclusive_sub_path_to_topic_name(sub_path2)
    } else {
        sub_path2
    };

    topic_name_regex_match(sub_topic_name1.clone(), sub_topic_name2.clone())
}

pub fn topic_name_regex_match(topic_name1: String, topic_name2: String) -> bool {
    base_topic_name_regex_match(topic_name1.clone(), topic_name2.clone())
        || base_topic_name_regex_match(topic_name2, topic_name1)
}

pub fn base_topic_name_regex_match(topic_name: String, regex_topic_name: String) -> bool {
    // Topic name perfect matching
    if topic_name == regex_topic_name {
        return true;
    }

    if regex_topic_name.contains("+") {
        let regex_str = regex_topic_name.replace("+", "[^+*/]+");
        let regex = Regex::new(&regex_str.to_string()).unwrap();
        return regex.is_match(&topic_name);
    }

    if regex_topic_name.contains("#") {
        if regex_topic_name.split("/").last().unwrap() != "#" {
            return false;
        }
        let regex_str = regex_topic_name.replace("#", "[^+#]+");
        let regex = Regex::new(&regex_str.to_string()).unwrap();
        return regex.is_match(&topic_name);
    }

    false
}
