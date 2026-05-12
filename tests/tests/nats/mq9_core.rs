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
    use async_nats::Client;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{
        DeliverPolicy, MailboxCreateReply, MailboxCreateReq, MsgAckReply, MsgAckReq, MsgFetchReply,
        MsgFetchReq, MsgQueryReply, MsgSendReply,
    };
    use std::time::Duration;
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

    async fn fetch(
        client: &Client,
        mail_address: &str,
        group_name: &str,
        num_msgs: u32,
    ) -> MsgFetchReply {
        let req = MsgFetchReq {
            group_name: Some(group_name.to_string()),
            deliver: DeliverPolicy::Earliest,
            from_time: None,
            from_id: None,
            force_deliver: None,
            config: Some(mq9_core::protocol::MsgFetchConfig {
                num_msgs: Some(num_msgs),
                max_wait_ms: None,
            }),
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgFetch {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, payload).await
    }

    async fn ack(
        client: &Client,
        mail_address: &str,
        group_name: &str,
        msg_id: u64,
    ) -> MsgAckReply {
        let req = MsgAckReq {
            group_name: group_name.to_string(),
            mail_address: mail_address.to_string(),
            msg_id,
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgAck {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, payload).await
    }

    #[tokio::test]
    async fn test_mailbox_query() {
        let client = nats_connect().await;

        // ── 1. send to non-existent mail → error ─────────────────────────────
        let fake_id = format!("nonexistent{}", unique_id());
        let subject = Mq9Command::MsgSend {
            mail_address: fake_id.clone(),
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
            name: Some(format!("test{}", unique_id().to_lowercase())),
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
    async fn test_mailbox_send_fetch_base() {
        let client = nats_connect().await;
        let group_name = format!("grp-{}", unique_id());

        // ── 1. create mail ────────────────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", unique_id().to_lowercase())),
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
            let payload_str = format!("fetch-msg-{}-{}", i, unique_id());
            let subject = Mq9Command::MsgSend {
                mail_address: mail_address.clone(),
            }
            .to_subject();
            let reply: MsgSendReply =
                request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(reply.error.is_empty(), "pub {}: {}", i, reply.error);
            sent_payloads.push(payload_str);
        }

        // ── 3. fetch all 10 messages ──────────────────────────────────────────
        let fetch_reply = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_reply.error.is_empty(),
            "fetch error: {}",
            fetch_reply.error
        );
        assert_eq!(
            fetch_reply.messages.len(),
            10,
            "expected 10 messages, got {}",
            fetch_reply.messages.len()
        );

        let received: Vec<String> = fetch_reply
            .messages
            .iter()
            .map(|m| m.payload.clone())
            .collect();
        for sent in &sent_payloads {
            assert!(
                received.iter().any(|r| r == sent),
                "payload '{}' not found in fetch reply",
                sent
            );
        }

        // ── 4. ack the last message ───────────────────────────────────────────
        let last_msg_id = fetch_reply.messages.last().unwrap().msg_id;
        let ack_reply = ack(&client, &mail_address, &group_name, last_msg_id).await;
        assert!(ack_reply.error.is_empty(), "ack error: {}", ack_reply.error);

        // ── 5. fetch again → empty (all consumed) ────────────────────────────
        let fetch_reply2 = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_reply2.error.is_empty(),
            "second fetch error: {}",
            fetch_reply2.error
        );
        assert_eq!(
            fetch_reply2.messages.len(),
            0,
            "expected 0 messages after ack, got {}",
            fetch_reply2.messages.len()
        );
    }
}
