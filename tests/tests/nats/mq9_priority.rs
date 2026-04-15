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
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    async fn publish(
        client: &Client,
        mail_id: &str,
        priority: Priority,
        payload: &str,
    ) -> Mq9Reply {
        let subject = Mq9Command::MailboxMsg {
            mail_id: mail_id.to_string(),
            priority,
        }
        .to_subject();
        nats_request(client, subject, Bytes::from(payload.to_string())).await
    }

    // Messages sent with mixed priorities must be delivered in
    // Critical → Urgent → Normal order regardless of send order.
    #[tokio::test]
    async fn test_priority() {
        let client = nats_connect().await;

        // ── 1. create mail ────────────────────────────────────────────────────
        let req = CreateMailboxReq {
            ttl: None,
            public: false,
            name: None,
            prefix: None,
            desc: "priority test mail".to_string(),
        };
        let reply = create_mail(&client, &req).await;
        assert!(!reply.is_error(), "create mail error: {}", reply.error);
        let mail_id = reply.mail_id.unwrap();

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
            let reply = publish(&client, &mail_id, priority.clone(), &tag).await;
            assert!(
                !reply.is_error(),
                "pub '{}' error: {}",
                payload,
                reply.error
            );
        }

        // ── 3. subscribe to mail ──────────────────────────────────────────────
        let sub_subject = Mq9Command::MailboxSub {
            mail_id: mail_id.clone(),
        }
        .to_subject();
        let mut sub = client.subscribe(sub_subject).await.unwrap();

        // ── 4. collect 10 messages with timeout ───────────────────────────────
        let mut received: Vec<String> = Vec::with_capacity(10);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while received.len() < 10 {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, sub.next()).await {
                Ok(Some(msg)) => {
                    let payload = String::from_utf8_lossy(&msg.payload).to_string();
                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    println!("[RECV {}ms] {}", ts, payload);
                    received.push(payload);
                }
                _ => break,
            }
        }

        assert_eq!(
            received.len(),
            10,
            "expected 10 messages, got {}",
            received.len()
        );

        // ── 5. verify priority order: all Critical before Urgent before Normal ─
        // Each received payload starts with the priority label we embedded above.
        fn priority_rank(payload: &str) -> u8 {
            if payload.starts_with("critical") {
                0
            } else if payload.starts_with("urgent") {
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
