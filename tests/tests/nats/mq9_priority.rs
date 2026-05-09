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
    use metadata_struct::mq9::Priority;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{
        DeliverPolicy, MailboxCreateReply, MailboxCreateReq, MsgFetchConfig, MsgFetchReply,
        MsgFetchReq, MsgSendReply,
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

    async fn publish(
        client: &Client,
        mail_address: &str,
        priority: Priority,
        payload: &str,
    ) -> MsgSendReply {
        let subject = Mq9Command::MsgSend {
            mail_address: mail_address.to_string(),
            priority,
        }
        .to_subject();
        request(client, subject, Bytes::from(payload.to_string())).await
    }

    async fn fetch(
        client: &Client,
        mail_address: &str,
        group_name: &str,
        num_msgs: u32,
    ) -> MsgFetchReply {
        let req = MsgFetchReq {
            group_name: group_name.to_string(),
            deliver: DeliverPolicy::Earliest,
            from_time: None,
            from_id: None,
            force_deliver: None,
            config: Some(MsgFetchConfig {
                num_msgs: Some(num_msgs),
            }),
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgFetch {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        request(client, subject, payload).await
    }

    // Messages sent with mixed priorities must be delivered in
    // Critical → Urgent → Normal order regardless of send order.
    #[tokio::test]
    async fn test_priority() {
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

        // ── 2. publish 10 messages with mixed priorities ──────────────────────
        // Send order: Normal, Critical, Urgent, ... (deliberately scrambled)
        // Expected receive order: all Critical first, then Urgent, then Normal.
        let msgs: Vec<(&str, Priority)> = vec![
            ("normal-1", Priority::Normal),
            ("critical-1", Priority::Critical),
            ("urgent-1", Priority::Urgent),
            ("normal-2", Priority::Normal),
            ("critical-2", Priority::Critical),
            ("urgent-2", Priority::Urgent),
            ("normal-3", Priority::Normal),
            ("critical-3", Priority::Critical),
            ("urgent-3", Priority::Urgent),
            ("normal-4", Priority::Normal),
        ];

        for (payload, priority) in &msgs {
            let tag = format!("[{}] {}-{}", priority, payload, unique_id());
            println!("[SEND] {}", tag);
            let reply = publish(&client, &mail_address, priority.clone(), &tag).await;
            assert!(
                reply.error.is_empty(),
                "pub '{}' error: {}",
                payload,
                reply.error
            );
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

        for (i, payload) in received.iter().enumerate() {
            println!("[RECV {}] {}", i, payload);
        }

        // ── 4. verify priority order: all Critical before Urgent before Normal ─
        fn priority_rank(payload: &str) -> u8 {
            if payload.contains("[critical]") || payload.contains("critical-") {
                0
            } else if payload.contains("[urgent]") || payload.contains("urgent-") {
                1
            } else {
                2
            }
        }

        for window in received.windows(2) {
            let rank_a = priority_rank(&window[0]);
            let rank_b = priority_rank(&window[1]);
            assert!(
                rank_a <= rank_b,
                "priority order violated: '{}' (rank {}) came before '{}' (rank {})",
                window[0],
                rank_a,
                window[1],
                rank_b
            );
        }
    }
}
