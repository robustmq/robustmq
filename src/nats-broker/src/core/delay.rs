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

use std::sync::Arc;

use bytes::Bytes;
use common_base::tools::now_second;
use delay_message::manager::{
    DelayMessageManager, DELAY_MESSAGE_FLAG, DELAY_MESSAGE_RECV_MS, DELAY_MESSAGE_TARGET_MS,
};
use metadata_struct::storage::adapter_record::{AdapterWriteRecord, RecordHeader};

use crate::core::error::NatsBrokerError;

/// Saves a delay message with metadata in UserProperties.
pub async fn save_delay_message(
    delay_message_manager: &Arc<DelayMessageManager>,
    tenant: &str,
    subject: &str,
    payload: &Bytes,
    delay_secs: u64,
) -> Result<Option<String>, NatsBrokerError> {
    let recv_time = now_second();
    let trigger_time = now_second() + delay_secs;

    let headers = vec![
        RecordHeader {
            name: DELAY_MESSAGE_FLAG.to_string(),
            value: "true".to_string(),
        },
        RecordHeader {
            name: DELAY_MESSAGE_RECV_MS.to_string(),
            value: recv_time.to_string(),
        },
        RecordHeader {
            name: DELAY_MESSAGE_TARGET_MS.to_string(),
            value: trigger_time.to_string(),
        },
    ];

    let record = AdapterWriteRecord::new(subject.to_string(), payload.clone()).with_header(headers);

    delay_message_manager
        .send(tenant, subject, trigger_time, record)
        .await?;

    Ok(None)
}
