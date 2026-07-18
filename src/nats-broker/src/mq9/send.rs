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
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mq9::Priority;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataMq9};
use mq9_core::protocol::MsgSendReply;
use storage_adapter::priority::storage_priority_tag;

const HEADER_MSG_KEY: &str = "mq9-key";
const HEADER_DELAY: &str = "mq9-delay";
const HEADER_TTL: &str = "mq9-ttl";
const HEADER_TAGS: &str = "mq9-tags";
const HEADER_PRIORITY: &str = "mq9-priority";

/// Parsed mq9-specific headers from a NATS HMSG header block.
pub struct Mq9Headers {
    /// `mq9-key`: dedup/compaction key for this message.
    pub msg_key: Option<String>,
    /// `mq9-delay`: seconds to delay delivery.
    pub delay_secs: Option<u64>,
    /// `mq9-ttl`: message-level TTL in seconds; expires at send_time + ttl.
    pub ttl_secs: Option<u64>,
    /// `mq9-tags`: comma-separated user tags, e.g. `billing,urgent,vip`.
    pub tags: Vec<String>,
    /// `mq9-priority`: `normal` | `urgent` | `critical`. Defaults to `normal`.
    pub priority: Priority,
}

/// Parse the raw NATS header block into `Mq9Headers`.
///
/// Format: `NATS/1.0\r\nKey: Value\r\n...\r\n`
fn parse_mq9_headers(raw: &Bytes) -> Mq9Headers {
    let mut msg_key = None;
    let mut delay_secs = None;
    let mut ttl_secs = None;
    let mut tags = vec![];
    let mut priority = Priority::Normal;

    let text = match std::str::from_utf8(raw) {
        Ok(s) => s,
        Err(_) => {
            return Mq9Headers {
                msg_key,
                delay_secs,
                ttl_secs: None,
                tags: vec![],
                priority,
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
                HEADER_TTL => ttl_secs = val.trim().parse().ok(),
                HEADER_TAGS => {
                    tags = val
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                HEADER_PRIORITY => {
                    priority = Priority::parse(val.trim()).unwrap_or(Priority::Normal);
                }
                _ => {}
            }
        }
    }

    Mq9Headers {
        msg_key,
        delay_secs,
        ttl_secs,
        tags,
        priority,
    }
}

pub async fn process_send(
    ctx: &NatsProcessContext,
    mail_address: &str,
    headers: &Option<Bytes>,
    reply_to: Option<&str>,
    payload: &Bytes,
) -> Result<MsgSendReply, NatsBrokerError> {
    let tenant = get_tenant();

    if ctx.cache_manager.get_mail(&tenant, mail_address).is_none() {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} does not exist",
            mail_address
        )));
    }

    let mq9_headers = headers.as_ref().map(parse_mq9_headers);
    let priority = mq9_headers
        .as_ref()
        .map(|h| h.priority.clone())
        .unwrap_or(Priority::Normal);

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

    let mut system_tags = build_message_tag(&tenant, mail_address, &priority);
    if let Some(h) = &mq9_headers {
        system_tags.extend(
            h.tags
                .iter()
                .map(|t| super::scoped_tag(&tenant, mail_address, t)),
        );
    }

    let mut record = AdapterWriteRecord::new(mail_address.to_string(), payload.clone())
        .with_tags(system_tags)
        .with_protocol_data(Some(StorageRecordProtocolData {
            mq9: Some(StorageRecordProtocolDataMq9 {
                priority: priority.to_string(),
                header: headers.clone(),
                reply_to: reply_to.map(|rp| rp.to_string()),
            }),
            ..Default::default()
        }));

    if let Some(h) = &mq9_headers {
        if let Some(key) = &h.msg_key {
            record = record.with_key(super::scoped_key(&tenant, mail_address, key));
        }
        if let Some(ttl) = h.ttl_secs {
            record = record.with_expire_at(now_second() + ttl);
        }
    }

    // send delay message
    if let Some(h) = &mq9_headers {
        if let Some(delay_secs) = h.delay_secs {
            save_delay_message(
                &ctx.delay_message_manager,
                &tenant,
                mail_address,
                &record,
                delay_secs,
            )
            .await?;
            return Ok(MsgSendReply {
                error: String::new(),
                msg_id: -1,
            });
        }
    }

    let storage = MessageStorage::new(ctx.storage_driver_manager.clone());

    // The mailbox write consumes the record by value. Forks need a clone
    // of the *pre-write* record so they see the same payload/tags/headers
    // the consumer will see. The fork path is inline and runs only after
    // the mailbox write succeeds.
    let user_tags: Vec<String> = mq9_headers
        .as_ref()
        .map(|h| h.tags.clone())
        .unwrap_or_default();
    let forked_source = ctx
        .cache_manager
        .match_forward_rules(&tenant, mail_address, &user_tags, &priority)
        .map(|rules| (record.clone(), rules));

    let offsets = storage.write(&tenant, mail_address, vec![record]).await?;

    let offset = offsets.into_iter().next().ok_or_else(|| {
        NatsBrokerError::CommonError(format!(
            "write to mailbox {} failed: no offset returned",
            mail_address
        ))
    })?;

    // Inline fork — direct awaited `MessageStorage::write` per matched rule.
    // No channels, no worker pool. `on_failure` decides per-rule whether
    // a fork failure should drop+log or fail the send. See forward.rs for
    // the full rationale.
    if let Some((src_record, rules)) = forked_source {
        crate::mq9::forward::fork_write(
            &storage,
            &tenant,
            mail_address,
            offset,
            &src_record,
            &rules,
        )
        .await?;
    }

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
        let raw = Bytes::from(
            "NATS/1.0\r\nmq9-key: k1\r\nmq9-delay: 30\r\nmq9-ttl: 60\r\nmq9-tags: billing,urgent,vip\r\nmq9-priority: critical\r\n\r\n",
        );
        let h = parse_mq9_headers(&raw);
        assert_eq!(h.msg_key.as_deref(), Some("k1"));
        assert_eq!(h.delay_secs, Some(30));
        assert_eq!(h.ttl_secs, Some(60));
        assert_eq!(h.tags, vec!["billing", "urgent", "vip"]);
        assert_eq!(h.priority, Priority::Critical);
    }

    #[test]
    fn test_parse_mq9_tags_with_spaces() {
        let raw = Bytes::from("NATS/1.0\r\nmq9-tags: a , b , c\r\n\r\n");
        let h = parse_mq9_headers(&raw);
        assert_eq!(h.tags, vec!["a", "b", "c"]);
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
