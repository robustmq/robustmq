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
        MsgFetchReq, MsgQueryReply, MsgSendReply,
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

    /// Send a message with `mq9-ttl: {ttl_secs}` header.
    async fn send_with_ttl(
        client: &Client,
        mail_address: &str,
        ttl_secs: u64,
        payload: &str,
    ) -> MsgSendReply {
        let subject = Mq9Command::MsgSend {
            mail_address: mail_address.to_string(),
        }
        .to_subject();

        let mut headers = HeaderMap::new();
        headers.insert("mq9-ttl", ttl_secs.to_string().as_str());

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

    async fn query_all(client: &Client, mail_address: &str) -> MsgQueryReply {
        let subject = Mq9Command::MsgQuery {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, Bytes::new()).await
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

    // A message with mq9-ttl is immediately visible, then disappears after TTL expires.
    #[tokio::test]
    async fn test_msg_ttl() {
        let client = nats_connect().await;
        let group_name = format!("grp-{}", unique_id());
        const TTL_SECS: u64 = 15;

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

        // ── 2. send a message with 15s TTL ────────────────────────────────────
        let payload = format!("ttl-msg-{}", unique_id());
        let send_reply = send_with_ttl(&client, &mail_address, TTL_SECS, &payload).await;
        assert!(
            send_reply.error.is_empty(),
            "send error: {}",
            send_reply.error
        );
        assert!(send_reply.msg_id >= 0, "expected valid msg_id");

        // ── 3. query immediately → message is visible ─────────────────────────
        let query_before = query_all(&client, &mail_address).await;
        assert!(
            query_before.error.is_empty(),
            "query error: {}",
            query_before.error
        );
        assert_eq!(
            query_before.messages.len(),
            1,
            "expected 1 message before TTL expiry, got {}",
            query_before.messages.len()
        );
        assert_eq!(
            query_before.messages[0].payload, payload,
            "payload mismatch before TTL: expected '{}', got '{}'",
            payload, query_before.messages[0].payload
        );

        // fetch also sees the message
        let fetch_before = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_before.error.is_empty(),
            "fetch error: {}",
            fetch_before.error
        );
        assert_eq!(
            fetch_before.messages.len(),
            1,
            "expected 1 message via fetch before TTL expiry, got {}",
            fetch_before.messages.len()
        );

        // ── 4. wait for TTL to expire ─────────────────────────────────────────
        println!("waiting {}s for TTL to expire...", TTL_SECS + 5);
        sleep(Duration::from_secs(TTL_SECS + 5)).await;

        // ── 5. query after TTL → message is gone ─────────────────────────────
        let query_after = query_all(&client, &mail_address).await;
        assert!(
            query_after.error.is_empty(),
            "query error: {}",
            query_after.error
        );
        assert_eq!(
            query_after.messages.len(),
            0,
            "expected 0 messages after TTL expiry, got {}",
            query_after.messages.len()
        );

        // fetch also returns nothing after TTL
        let fetch_after = fetch(&client, &mail_address, &group_name, 10).await;
        assert!(
            fetch_after.error.is_empty(),
            "fetch error: {}",
            fetch_after.error
        );
        assert_eq!(
            fetch_after.messages.len(),
            0,
            "expected 0 messages via fetch after TTL expiry, got {}",
            fetch_after.messages.len()
        );
    }
}
