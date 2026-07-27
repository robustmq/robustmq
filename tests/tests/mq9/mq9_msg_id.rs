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
        DeliverPolicy, MailboxCreateReply, MailboxCreateReq, MsgFetchConfig, MsgFetchReply,
        MsgFetchReq, MsgQueryReply, MsgQueryReq, MsgSendReply,
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

    /// Send a message with an `mq9-key` header for dedup/compaction.
    async fn send_with_key(
        client: &Client,
        mail_address: &str,
        key: &str,
        payload: &str,
    ) -> MsgSendReply {
        let subject = Mq9Command::MsgSend {
            mail_address: mail_address.to_string(),
        }
        .to_subject();

        let mut headers = HeaderMap::new();
        headers.insert("mq9-key", key);

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

    async fn query_by_key(client: &Client, mail_address: &str, key: &str) -> MsgQueryReply {
        let req = MsgQueryReq {
            key: Some(key.to_string()),
            ..Default::default()
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgQuery {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, payload).await
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
            config: Some(MsgFetchConfig {
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

    // Send multiple messages with the same key; only the latest one should be
    // visible via QUERY (by key) and FETCH.
    #[tokio::test]
    async fn test_msg_key_dedup() {
        let client = nats_connect().await;
        let group_name = format!("grp-{}", unique_id());

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

        // ── 2. send 5 messages with the same key ──────────────────────────────
        let key = format!("dedup-key-{}", unique_id());
        let last_payload = format!("payload-last-{}", unique_id());

        for i in 0..4usize {
            let payload = format!("payload-{}-{}", i, unique_id());
            let reply = send_with_key(&client, &mail_address, &key, &payload).await;
            assert!(reply.error.is_empty(), "send {} error: {}", i, reply.error);
        }
        // last message — this is the one that should survive
        let reply = send_with_key(&client, &mail_address, &key, &last_payload).await;
        assert!(reply.error.is_empty(), "send last error: {}", reply.error);

        // ── 3. query by key → only 1 message, the last one ───────────────────
        let query_reply = query_by_key(&client, &mail_address, &key).await;
        assert!(
            query_reply.error.is_empty(),
            "query error: {}",
            query_reply.error
        );
        assert_eq!(
            query_reply.messages.len(),
            1,
            "expected exactly 1 message for key, got {}",
            query_reply.messages.len()
        );
        assert_eq!(
            query_reply.messages[0].payload, last_payload,
            "query: expected last payload, got '{}'",
            query_reply.messages[0].payload
        );

        // ── 4. fetch → only 1 message, the last one ───────────────────────────
        let fetch_reply = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_reply.error.is_empty(),
            "fetch error: {}",
            fetch_reply.error
        );
        assert_eq!(
            fetch_reply.messages.len(),
            1,
            "expected exactly 1 message via fetch, got {}",
            fetch_reply.messages.len()
        );
        assert_eq!(
            fetch_reply.messages[0].payload, last_payload,
            "fetch: expected last payload, got '{}'",
            fetch_reply.messages[0].payload
        );
    }

    // Messages with different keys are independent — each key keeps its own latest.
    #[tokio::test]
    async fn test_msg_key_independent() {
        let client = nats_connect().await;
        let group_name = format!("grp-{}", unique_id());

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

        // ── 2. send 3 messages per key (key-a and key-b) ─────────────────────
        let key_a = format!("key-a-{}", unique_id());
        let key_b = format!("key-b-{}", unique_id());
        let last_a = format!("last-a-{}", unique_id());
        let last_b = format!("last-b-{}", unique_id());

        for i in 0..2usize {
            let r = send_with_key(&client, &mail_address, &key_a, &format!("a-{}", i)).await;
            assert!(r.error.is_empty(), "send a-{}: {}", i, r.error);
            let r = send_with_key(&client, &mail_address, &key_b, &format!("b-{}", i)).await;
            assert!(r.error.is_empty(), "send b-{}: {}", i, r.error);
        }
        let r = send_with_key(&client, &mail_address, &key_a, &last_a).await;
        assert!(r.error.is_empty(), "send last-a: {}", r.error);
        let r = send_with_key(&client, &mail_address, &key_b, &last_b).await;
        assert!(r.error.is_empty(), "send last-b: {}", r.error);

        // ── 3. query each key → 1 message each, the respective last payload ───
        let qa = query_by_key(&client, &mail_address, &key_a).await;
        assert!(qa.error.is_empty(), "query key-a error: {}", qa.error);
        assert_eq!(qa.messages.len(), 1, "key-a: expected 1 message");
        assert_eq!(qa.messages[0].payload, last_a, "key-a: wrong payload");

        let qb = query_by_key(&client, &mail_address, &key_b).await;
        assert!(qb.error.is_empty(), "query key-b error: {}", qb.error);
        assert_eq!(qb.messages.len(), 1, "key-b: expected 1 message");
        assert_eq!(qb.messages[0].payload, last_b, "key-b: wrong payload");

        // ── 4. fetch → 2 messages total (one per key, each the latest) ────────
        let fetch_reply = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_reply.error.is_empty(),
            "fetch error: {}",
            fetch_reply.error
        );
        assert_eq!(
            fetch_reply.messages.len(),
            2,
            "expected 2 messages (one per key), got {}",
            fetch_reply.messages.len()
        );
        let payloads: Vec<&str> = fetch_reply
            .messages
            .iter()
            .map(|m| m.payload.as_str())
            .collect();
        assert!(payloads.contains(&last_a.as_str()), "fetch missing last-a");
        assert!(payloads.contains(&last_b.as_str()), "fetch missing last-b");
    }
}
