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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_nats::{Client, HeaderMap};
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{
        MailboxCreateReply, MailboxCreateReq, MsgQueryReply, MsgQueryReq, MsgSendReply,
    };
    use tokio::time::sleep;

    use crate::nats::common::nats_connect;

    async fn request<T: serde::de::DeserializeOwned>(
        client: &Client,
        subject: String,
        payload: Bytes,
    ) -> T {
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<T>(&msg.payload).unwrap_or_else(|_| {
            panic!(
                "failed to parse reply, raw: {}",
                String::from_utf8_lossy(&msg.payload)
            )
        })
    }

    async fn create_mail(client: &Client, req: &MailboxCreateReq) -> MailboxCreateReply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        request(client, Mq9Command::MailboxCreate.to_subject(), payload).await
    }

    async fn send_with_tags(
        client: &Client,
        mail_address: &str,
        tags: &[&str],
        payload: &str,
    ) -> MsgSendReply {
        let subject = Mq9Command::MsgSend {
            mail_address: mail_address.to_string(),
        }
        .to_subject();

        let mut headers = HeaderMap::new();
        headers.insert("mq9-tags", tags.join(",").as_str());

        let msg = client
            .request_with_headers(subject, headers, Bytes::from(payload.to_string()))
            .await
            .unwrap();
        serde_json::from_slice::<MsgSendReply>(&msg.payload).unwrap_or_else(|_| {
            panic!(
                "failed to parse send reply, raw: {}",
                String::from_utf8_lossy(&msg.payload)
            )
        })
    }

    async fn query_by_tags(
        client: &Client,
        mail_address: &str,
        tags: Vec<String>,
    ) -> MsgQueryReply {
        let req = MsgQueryReq {
            tags: Some(tags),
            ..Default::default()
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgQuery {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, payload).await
    }

    // Messages sent with mq9-tags can be queried by tag.
    // Querying with a matching tag returns the message;
    // querying with a non-matching tag returns nothing.
    #[tokio::test]
    async fn test_msg_tags_query() {
        let client = nats_connect().await;

        // ── 1. create mailbox ─────────────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", unique_id().to_lowercase())),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&client, &req).await;
        assert!(reply.error.is_empty(), "create mail error: {}", reply.error);
        let mail_address = reply.mail_address;

        sleep(Duration::from_secs(3)).await;

        // ── 2. send messages with different tags ──────────────────────────────
        let payload_billing = format!("billing-msg-{}", unique_id());
        let payload_urgent = format!("urgent-msg-{}", unique_id());
        let payload_both = format!("both-msg-{}", unique_id());

        let r = send_with_tags(&client, &mail_address, &["billing"], &payload_billing).await;
        assert!(r.error.is_empty(), "send billing error: {}", r.error);

        let r = send_with_tags(&client, &mail_address, &["urgent"], &payload_urgent).await;
        assert!(r.error.is_empty(), "send urgent error: {}", r.error);

        let r = send_with_tags(
            &client,
            &mail_address,
            &["billing", "urgent"],
            &payload_both,
        )
        .await;
        assert!(r.error.is_empty(), "send both error: {}", r.error);

        // ── 3. query with matching tag → returns messages that carry that tag ─
        let reply = query_by_tags(&client, &mail_address, vec!["billing".to_string()]).await;
        assert!(
            reply.error.is_empty(),
            "query billing error: {}",
            reply.error
        );
        assert_eq!(
            reply.messages.len(),
            2,
            "expected 2 messages with tag 'billing', got {}",
            reply.messages.len()
        );
        let payloads: Vec<&str> = reply.messages.iter().map(|m| m.payload.as_str()).collect();
        assert!(
            payloads.contains(&payload_billing.as_str()),
            "missing billing-only message"
        );
        assert!(
            payloads.contains(&payload_both.as_str()),
            "missing billing+urgent message"
        );

        // ── 4. query with non-matching tag → returns nothing ──────────────────
        let reply =
            query_by_tags(&client, &mail_address, vec!["nonexistent-tag".to_string()]).await;
        assert!(
            reply.error.is_empty(),
            "query nonexistent error: {}",
            reply.error
        );
        assert_eq!(
            reply.messages.len(),
            0,
            "expected 0 messages for unknown tag, got {}",
            reply.messages.len()
        );
    }

    // Untagged messages must not appear in tag-filtered queries.
    #[tokio::test]
    async fn test_untagged_messages_excluded() {
        let client = nats_connect().await;

        // ── 1. create mailbox ─────────────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", unique_id().to_lowercase())),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&client, &req).await;
        assert!(reply.error.is_empty(), "create mail error: {}", reply.error);
        let mail_address = reply.mail_address;

        sleep(Duration::from_secs(3)).await;

        // ── 2. send one tagged and one untagged message ───────────────────────
        let payload_tagged = format!("tagged-{}", unique_id());
        let payload_untagged = format!("untagged-{}", unique_id());

        let r = send_with_tags(&client, &mail_address, &["vip"], &payload_tagged).await;
        assert!(r.error.is_empty(), "send tagged error: {}", r.error);

        let untagged_subject = Mq9Command::MsgSend {
            mail_address: mail_address.clone(),
        }
        .to_subject();
        let msg = client
            .request(untagged_subject, Bytes::from(payload_untagged.clone()))
            .await
            .unwrap();
        let r = serde_json::from_slice::<MsgSendReply>(&msg.payload).unwrap_or_else(|_| {
            panic!(
                "failed to parse untagged send reply, raw: {}",
                String::from_utf8_lossy(&msg.payload)
            )
        });
        assert!(r.error.is_empty(), "send untagged error: {}", r.error);

        // ── 3. query by tag "vip" → only the tagged message ──────────────────
        let reply = query_by_tags(&client, &mail_address, vec!["vip".to_string()]).await;
        assert!(reply.error.is_empty(), "query vip error: {}", reply.error);
        assert_eq!(
            reply.messages.len(),
            1,
            "expected 1 message with tag 'vip', got {}",
            reply.messages.len()
        );
        assert_eq!(
            reply.messages[0].payload, payload_tagged,
            "wrong payload returned for tag query"
        );

        // ── 4. query by wrong tag → no messages ───────────────────────────────
        let reply = query_by_tags(&client, &mail_address, vec!["wrong-tag".to_string()]).await;
        assert!(
            reply.error.is_empty(),
            "query wrong-tag error: {}",
            reply.error
        );
        assert_eq!(
            reply.messages.len(),
            0,
            "expected 0 messages for wrong tag, got {}",
            reply.messages.len()
        );
    }
}
