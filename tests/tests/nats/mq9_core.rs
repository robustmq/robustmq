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

    use async_nats::Client;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use futures::StreamExt;
    use metadata_struct::mq9::Priority;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{MailboxCreateReply, MailboxCreateReq, MsgQueryReply, MsgSendReply};
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

    #[tokio::test]
    async fn test_mailbox_core() {
        let client = nats_connect().await;

        // ── 1. send to non-existent mail → error ─────────────────────────────
        let fake_id = format!("nonexistent{}", unique_id());
        let subject = Mq9Command::MsgSend {
            mail_address: fake_id.clone(),
            priority: Priority::Normal,
        }
        .to_subject();
        let reply: MsgSendReply = request(&client, subject, Bytes::from("hello")).await;
        assert!(!reply.error.is_empty(), "expected an error reply");
        assert!(
            reply.error.contains("does not exist"),
            "expected 'does not exist' in error, got: {}",
            reply.error
        );

        // ── 2. create mail → success ──────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", &unique_id().to_lowercase()[..8])),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&client, &req).await;
        assert!(reply.error.is_empty(), "unexpected error: {}", reply.error);
        assert!(
            !reply.mail_address.is_empty(),
            "mail_address should not be empty"
        );
        let mail_address = reply.mail_address;

        sleep(Duration::from_secs(3)).await;
        // ── 3. send 10 messages → all succeed ────────────────────────────────
        let mut sent_payloads = Vec::with_capacity(10);
        for i in 0..10usize {
            let payload_str = format!("message-{}-{}", i, unique_id());
            let subject = Mq9Command::MsgSend {
                mail_address: mail_address.clone(),
                priority: Priority::Normal,
            }
            .to_subject();
            let reply: MsgSendReply =
                request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(
                reply.error.is_empty(),
                "msg {}: unexpected error: {}",
                i,
                reply.error
            );
            assert!(
                reply.msg_id > 0 || i == 0,
                "msg_id should be a valid offset"
            );
            sent_payloads.push(payload_str);
        }

        // ── 4. query messages → get back all 10 ──────────────────────────────
        let query_subject = Mq9Command::MsgQuery {
            mail_address: mail_address.clone(),
        }
        .to_subject();
        let reply: MsgQueryReply = request(&client, query_subject, Bytes::new()).await;
        assert!(reply.error.is_empty(), "unexpected error: {}", reply.error);
        assert_eq!(
            reply.messages.len(),
            10,
            "expected 10 messages, got {}",
            reply.messages.len()
        );
        for sent in &sent_payloads {
            assert!(
                reply.messages.iter().any(|m| &m.payload == sent),
                "payload '{}' not found in query reply",
                sent
            );
        }
    }

    #[tokio::test]
    async fn test_mailbox_send_sub() {
        let client = nats_connect().await;

        // ── 1. create mail ────────────────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", &unique_id().to_lowercase()[..8])),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&client, &req).await;
        assert!(reply.error.is_empty(), "create mail error: {}", reply.error);
        let mail_address = reply.mail_address;

        sleep(Duration::from_secs(3)).await;
        // ── 2. publish 10 messages ────────────────────────────────────────────
        let mut sent_payloads = Vec::with_capacity(10);
        for i in 0..10usize {
            let payload_str = format!("sub-msg-{}-{}", i, unique_id());
            let subject = Mq9Command::MsgSend {
                mail_address: mail_address.clone(),
                priority: Priority::Normal,
            }
            .to_subject();
            let reply: MsgSendReply =
                request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(reply.error.is_empty(), "pub {}: {}", i, reply.error);
            sent_payloads.push(payload_str);
        }

        // ── 3. subscribe to mail ──────────────────────────────────────────────
        let sub_subject = Mq9Command::MsgSub {
            mail_address: mail_address.clone(),
        }
        .to_subject();
        let mut sub = client.subscribe(sub_subject).await.unwrap();

        // ── 4. collect 10 messages with timeout ───────────────────────────────
        let mut received = Vec::with_capacity(10);
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        while received.len() < 10 {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, sub.next()).await {
                Ok(Some(msg)) => received.push(String::from_utf8_lossy(&msg.payload).to_string()),
                _ => break,
            }
        }

        assert_eq!(
            received.len(),
            10,
            "expected 10 messages, got {}",
            received.len()
        );
        for sent in &sent_payloads {
            assert!(
                received.iter().any(|r| r == sent),
                "payload '{}' not found in received messages",
                sent
            );
        }
    }
}
