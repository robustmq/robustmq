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

use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use protocol::mqtt::common::RetainHandling;

use crate::subscribe::common::Subscriber;

// No Local
// The only values available for No Local are 0 and 1, where 1 means that the server cannot forward the message to the client that posted it, and 0 is the opposite.
pub fn message_is_same_client(subscriber: &Subscriber, record: &StorageRecord) -> bool {
    if let Some(protocol_data) = record.protocol_data.clone() {
        if let Some(mqtt_data) = protocol_data.mqtt {
            if subscriber.no_local && subscriber.client_id == mqtt_data.client_id {
                return true;
            }
        }
    }
    false
}

// Retain Handling
pub fn is_send_retain_msg_by_retain_handling(
    path: &str,
    retain_handling: &RetainHandling,
    is_new_subs: &DashMap<String, bool>,
) -> bool {
    if *retain_handling == RetainHandling::OnEverySubscribe {
        return true;
    }

    if *retain_handling == RetainHandling::Never {
        return false;
    }

    if *retain_handling == RetainHandling::OnNewSubscribe {
        let is_new_sub = if let Some(new_sub) = is_new_subs.get(path) {
            new_sub.to_owned()
        } else {
            true
        };

        if is_new_sub {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::core::sub_option::is_send_retain_msg_by_retain_handling;
    use dashmap::DashMap;
    use protocol::mqtt::common::RetainHandling;

    #[test]
    fn is_send_retain_msg_by_retain_handling_test() {
        let path = "path";

        // OnEverySubscribe → always send
        assert!(is_send_retain_msg_by_retain_handling(
            path,
            &RetainHandling::OnEverySubscribe,
            &DashMap::new()
        ));

        // Never → never send
        assert!(!is_send_retain_msg_by_retain_handling(
            path,
            &RetainHandling::Never,
            &DashMap::new()
        ));

        // OnNewSubscribe + is_new=true → send
        let is_new_subs = DashMap::new();
        is_new_subs.insert(path.to_string(), true);
        assert!(is_send_retain_msg_by_retain_handling(
            path,
            &RetainHandling::OnNewSubscribe,
            &is_new_subs
        ));

        // OnNewSubscribe + is_new=false → don't send
        let is_new_subs = DashMap::new();
        is_new_subs.insert(path.to_string(), false);
        assert!(!is_send_retain_msg_by_retain_handling(
            path,
            &RetainHandling::OnNewSubscribe,
            &is_new_subs
        ));

        // OnNewSubscribe + path not in map → treat as new, send
        assert!(is_send_retain_msg_by_retain_handling(
            path,
            &RetainHandling::OnNewSubscribe,
            &DashMap::new()
        ));
    }
}
