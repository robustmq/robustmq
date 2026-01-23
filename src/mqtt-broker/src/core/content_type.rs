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

use protocol::mqtt::common::{LastWill, LastWillProperties, Publish, PublishProperties};
use std::str::from_utf8;

pub fn payload_format_indicator_check_by_lastwill(
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
) -> bool {
    if let Some(will) = last_will {
        if let Some(properties) = last_will_properties {
            if let Some(payload_format_indicator) = properties.payload_format_indicator {
                return payload_format_indicator_check(&will.message, payload_format_indicator);
            }
        }
    }

    true
}

pub fn payload_format_indicator_check_by_publish(
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> bool {
    if let Some(properties) = publish_properties {
        if let Some(payload_format_indicator) = properties.payload_format_indicator {
            return payload_format_indicator_check(&publish.payload, payload_format_indicator);
        }
    }
    true
}

fn payload_format_indicator_check(payload: &[u8], payload_format_indicator: u8) -> bool {
    if payload_format_indicator == 1 && from_utf8(payload).is_err() {
        return false;
    }
    true
}

#[cfg(test)]
mod test {
    use crate::core::content_type::payload_format_indicator_check;

    #[tokio::test]
    async fn payload_format_indicator_check_test() {
        let valid_utf8_bytes = "Hello, world".as_bytes();
        let invalid_utf8_bytes = &[0xff, 0xfe, 0xfd];
        assert!(payload_format_indicator_check(valid_utf8_bytes, 0));
        assert!(payload_format_indicator_check(invalid_utf8_bytes, 0));

        assert!(payload_format_indicator_check(valid_utf8_bytes, 1));
        assert!(!payload_format_indicator_check(invalid_utf8_bytes, 1));
    }
}
