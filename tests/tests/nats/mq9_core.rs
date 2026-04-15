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
    use mq9_core::protocol::{CreateMailboxReq, Mq9Reply};
    use tokio::time::sleep;

    const NATS_ADDR: &str = "nats://127.0.0.1:4222";

    async fn nats_connect() -> Client {
        async_nats::connect(NATS_ADDR).await.unwrap()
    }

    async fn nats_request(client: &Client, subject: String, payload: Bytes) -> Mq9Reply {
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<Mq9Reply>(&msg.payload).unwrap_or_else(|_| {
            panic!(
                "failed to parse Mq9Reply, raw: {}",
                String::from_utf8_lossy(&msg.payload)
            )
        })
    }

    async fn create_mail(client: &Client, req: &CreateMailboxReq) -> Mq9Reply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        nats_request(client, Mq9Command::MailboxCreate.to_subject(), payload).await
    }

    #[tokio::test]
    async fn test_mailbox_core() {
        let client = nats_connect().await;

        // ── 1. send to non-existent mail → error ─────────────────────────────
        let fake_id = format!("nonexistent-{}", unique_id());
        let subject = Mq9Command::MailboxMsg {
            mail_id: fake_id.clone(),
            priority: Priority::Normal,
        }
        .to_subject();
        let reply = nats_request(&client, subject, Bytes::from("hello")).await;
        assert!(reply.is_error(), "expected an error reply");
        assert!(
            reply.error.contains("does not exist"),
            "expected 'does not exist' in error, got: {}",
            reply.error
        );

        // ── 2. create mail → success ──────────────────────────────────────────
        let req = CreateMailboxReq {
            ttl: None,
            public: false,
            name: None,
            desc: "mq9_core test mail".to_string(),
        };
        let reply = create_mail(&client, &req).await;
        assert!(!reply.is_error(), "unexpected error: {}", reply.error);
        assert!(
            !reply.mail_id.as_deref().unwrap_or("").is_empty(),
            "mail_id should not be empty"
        );
        assert!(reply.is_new.unwrap_or(false), "should be a new mailbox");
        let mail_id = reply.mail_id.unwrap();

        sleep(Duration::from_secs(3)).await;
        // ── 3. send 10 messages → all succeed ────────────────────────────────
        let mut sent_payloads = Vec::with_capacity(10);
        for i in 0..10usize {
            let payload_str = format!("message-{}-{}", i, unique_id());
            let subject = Mq9Command::MailboxMsg {
                mail_id: mail_id.clone(),
                priority: Priority::Normal,
            }
            .to_subject();
            let reply = nats_request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(
                !reply.is_error(),
                "msg {}: unexpected error: {}",
                i,
                reply.error
            );
            assert!(
                reply.msg_id.unwrap_or(0) > 0 || i == 0,
                "msg_id should be a valid offset"
            );
            sent_payloads.push(payload_str);
        }

        // ── 4. list messages → get back all 10 ───────────────────────────────
        let list_subject = Mq9Command::MailboxList {
            mail_id: mail_id.clone(),
        }
        .to_subject();
        let reply = nats_request(&client, list_subject, Bytes::new()).await;
        assert!(!reply.is_error(), "unexpected error: {}", reply.error);
        let messages = reply.messages.unwrap();
        assert_eq!(
            messages.len(),
            10,
            "expected 10 messages, got {}",
            messages.len()
        );
        for sent in &sent_payloads {
            assert!(
                messages.iter().any(|m| &m.payload == sent),
                "payload '{}' not found in list reply",
                sent
            );
        }
    }

    #[tokio::test]
    async fn test_mailbox_send_sub() {
        let client = nats_connect().await;

        // ── 1. create mail ────────────────────────────────────────────────────
        let req = CreateMailboxReq {
            ttl: None,
            public: false,
            name: None,
            desc: "send-sub test".to_string(),
        };
        let reply = create_mail(&client, &req).await;
        assert!(!reply.is_error(), "create mail error: {}", reply.error);
        let mail_id = reply.mail_id.unwrap();

        sleep(Duration::from_secs(3)).await;
        // ── 2. publish 10 messages ────────────────────────────────────────────
        let mut sent_payloads = Vec::with_capacity(10);
        for i in 0..10usize {
            let payload_str = format!("sub-msg-{}-{}", i, unique_id());
            let subject = Mq9Command::MailboxMsg {
                mail_id: mail_id.clone(),
                priority: Priority::Normal,
            }
            .to_subject();
            let reply = nats_request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(!reply.is_error(), "pub {}: {}", i, reply.error);
            sent_payloads.push(payload_str);
        }

        // ── 3. subscribe to mail ──────────────────────────────────────────────
        let sub_subject = Mq9Command::MailboxSub {
            mail_id: mail_id.clone(),
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
