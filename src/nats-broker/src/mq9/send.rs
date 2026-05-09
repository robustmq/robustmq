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

use crate::core::delay::save_delay_message;
use crate::core::error::NatsBrokerError;
use crate::core::subject::try_get_or_init_subject;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::nats::subscribe::subject_message_tag;
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mq9::Priority;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataMq9};
use mq9_core::protocol::MsgSendReply;
use mq9_core::public::is_system_mailbox;
use storage_adapter::priority::storage_priority_tag;

const HEADER_MSG_KEY: &str = "mq9-key";
const HEADER_DELAY: &str = "mq9-delay";

/// Parsed mq9-specific headers from a NATS HMSG header block.
pub struct Mq9Headers {
    /// `mq9-key`: dedup/compaction key for this message.
    pub msg_key: Option<String>,
    /// `mq9-delay`: seconds to delay delivery.
    pub delay_secs: Option<u64>,
}

/// Parse the raw NATS header block into `Mq9Headers`.
///
/// Format: `NATS/1.0\r\nKey: Value\r\n...\r\n`
fn parse_mq9_headers(raw: &Bytes) -> Mq9Headers {
    let mut msg_key = None;
    let mut delay_secs = None;

    let text = match std::str::from_utf8(raw) {
        Ok(s) => s,
        Err(_) => {
            return Mq9Headers {
                msg_key,
                delay_secs,
            }
        }
    };

    // Skip the status line ("NATS/1.0\r\n"), then parse each "Key: Value\r\n"
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((key, val)) = line.split_once(':') {
            match key.trim() {
                HEADER_MSG_KEY => msg_key = Some(val.trim().to_string()),
                HEADER_DELAY => delay_secs = val.trim().parse().ok(),
                _ => {}
            }
        }
    }

    Mq9Headers {
        msg_key,
        delay_secs,
    }
}

pub async fn process_send(
    ctx: &NatsProcessContext,
    mail_address: &str,
    priority: &Priority,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Result<MsgSendReply, NatsBrokerError> {
    let tenant = get_tenant();

    if is_system_mailbox(mail_address) {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox '{}' is reserved and cannot receive messages from clients",
            mail_address
        )));
    }

    if ctx.cache_manager.get_mail(&tenant, mail_address).is_none() {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} does not exist",
            mail_address
        )));
    }

    let mq9_headers = headers.as_ref().map(parse_mq9_headers);

    try_get_or_init_subject(
        &ctx.cache_manager,
        &ctx.storage_driver_manager,
        &ctx.client_pool,
        &ctx.subscribe_manager,
        &tenant,
        mail_address,
        true,
    )
    .await?;

    let mut record = AdapterWriteRecord::new(mail_address.to_string(), payload.clone())
        .with_tags(build_message_tag(&tenant, mail_address, priority))
        .with_protocol_data(Some(StorageRecordProtocolData {
            mq9: Some(StorageRecordProtocolDataMq9 {
                priority: priority.to_string(),
                header: headers.clone(),
            }),
            nats: None,
            mqtt: None,
        }));

    if let Some(h) = &mq9_headers {
        if let Some(key) = &h.msg_key {
            record = record.with_key(key);
        }
    }

    // send delay message
    if let Some(h) = &mq9_headers {
        if let Some(delay_secs) = h.delay_secs {
            save_delay_message(
                &ctx.delay_message_manager,
                &tenant,
                mail_address,
                payload,
                delay_secs,
            )
            .await?;
            return Ok(MsgSendReply {
                error: String::new(),
                msg_id: -1,
            });
        }
    }

    let offsets = MessageStorage::new(ctx.storage_driver_manager.clone())
        .write(&tenant, mail_address, vec![record])
        .await?;

    let offset = offsets.into_iter().next().ok_or_else(|| {
        NatsBrokerError::CommonError(format!(
            "write to mailbox {} failed: no offset returned",
            mail_address
        ))
    })?;
    Ok(MsgSendReply {
        error: String::new(),
        msg_id: offset as i64,
    })
}

fn build_message_tag(tenant: &str, mail_address: &str, priority: &Priority) -> Vec<String> {
    let subject_tag = subject_message_tag(tenant, mail_address);
    let subject_priority = storage_priority_tag(&subject_tag, priority);
    vec![subject_tag, subject_priority]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mq9_headers() {
        let raw = Bytes::from("NATS/1.0\r\nmq9-key: k1\r\nmq9-delay: 30\r\n\r\n");
        let h = parse_mq9_headers(&raw);
        assert_eq!(h.msg_key.as_deref(), Some("k1"));
        assert_eq!(h.delay_secs, Some(30));
    }

    #[test]
    fn test_parse_mq9_headers_partial() {
        let raw = Bytes::from("NATS/1.0\r\nmq9-key: mykey\r\n\r\n");
        let h = parse_mq9_headers(&raw);
        assert_eq!(h.msg_key.as_deref(), Some("mykey"));
        assert_eq!(h.delay_secs, None);
    }

    #[test]
    fn test_parse_mq9_headers_empty() {
        let raw = Bytes::from("NATS/1.0\r\n\r\n");
        let h = parse_mq9_headers(&raw);
        assert!(h.msg_key.is_none());
        assert!(h.delay_secs.is_none());
    }
}
