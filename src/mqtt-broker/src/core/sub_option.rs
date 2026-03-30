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
pub fn is_send_msg_by_bo_local(subscriber: &Subscriber, record: &StorageRecord) -> bool {
    if let Some(protocol_data) = record.protocol_data.clone() {
        if let Some(mqtt_data) = protocol_data.mqtt {
            if subscriber.no_local && subscriber.client_id == mqtt_data.client_id {
                return false;
            }
        }
    }
    true
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
    use bytes::Bytes;
    use dashmap::DashMap;
    use metadata_struct::storage::record::{
        StorageRecord, StorageRecordMetadata, StorageRecordProtocolData,
        StorageRecordProtocolDataMqtt,
    };
    use protocol::mqtt::common::{MqttProtocol, QoS, RetainHandling};

    use crate::core::sub_option::{is_send_msg_by_bo_local, is_send_retain_msg_by_retain_handling};
    use crate::subscribe::common::Subscriber;

    fn build_subscriber(client_id: &str, no_local: bool) -> Subscriber {
        Subscriber {
            client_id: client_id.to_string(),
            sub_path: "/test".to_string(),
            rewrite_sub_path: None,
            tenant: "default".to_string(),
            topic_name: "/test".to_string(),
            group_name: "g".to_string(),
            protocol: MqttProtocol::Mqtt5,
            qos: QoS::AtLeastOnce,
            no_local,
            preserve_retain: false,
            retain_forward_rule: RetainHandling::OnEverySubscribe,
            subscription_identifier: None,
            create_time: 0,
        }
    }

    fn build_record_with_client(client_id: &str) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata::build(0, "shard".to_string(), 0),
            protocol_data: Some(StorageRecordProtocolData {
                mqtt: Some(StorageRecordProtocolDataMqtt {
                    client_id: client_id.to_string(),
                    retain: false,
                    format_indicator: None,
                    response_topic: None,
                    correlation_data: None,
                    content_type: None,
                    expire_at: 0,
                }),
            }),
            data: Bytes::new(),
        }
    }

    #[test]
    fn is_send_msg_by_bo_local_test() {
        // no_local=true, same client_id → should NOT send
        assert!(!is_send_msg_by_bo_local(
            &build_subscriber("client_id", true),
            &build_record_with_client("client_id")
        ));

        // no_local=true, different client_id → should send
        assert!(is_send_msg_by_bo_local(
            &build_subscriber("client_id", true),
            &build_record_with_client("client_id_1")
        ));

        // no_local=false, same client_id → should send
        assert!(is_send_msg_by_bo_local(
            &build_subscriber("client_id", false),
            &build_record_with_client("client_id")
        ));

        // no_local=false, different client_id → should send
        assert!(is_send_msg_by_bo_local(
            &build_subscriber("client_id", false),
            &build_record_with_client("client_id_1")
        ));

        // no protocol_data → should send regardless of no_local
        let record_no_protocol = StorageRecord {
            metadata: StorageRecordMetadata::build(0, "shard".to_string(), 0),
            protocol_data: None,
            data: Bytes::new(),
        };
        assert!(is_send_msg_by_bo_local(
            &build_subscriber("client_id", true),
            &record_no_protocol
        ));
    }

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
