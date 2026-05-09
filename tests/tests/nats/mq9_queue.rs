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
    use metadata_struct::mq9::Priority;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{
        DeliverPolicy, MailboxCreateReply, MailboxCreateReq, MsgAckReply, MsgAckReq,
        MsgFetchConfig, MsgFetchReply, MsgFetchReq, MsgSendReply,
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

    // Two independent consumer groups each fetch all messages independently.
    // After ACK, re-fetch yields no new messages for that group.
    #[tokio::test]
    async fn test_mq9_fetch_ack() {
        let client = nats_connect().await;
        let group_a = format!("grp-a-{}", unique_id());
        let group_b = format!("grp-b-{}", unique_id());

        // ── 1. create mailbox ─────────────────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(format!("test{}", &unique_id().to_lowercase()[..8])),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&client, &req).await;
        assert!(reply.error.is_empty(), "create mail error: {}", reply.error);
        let mail_address = reply.mail_address;

        sleep(Duration::from_secs(3)).await;

        // ── 2. publish 7 messages ─────────────────────────────────────────────
        let mut sent_payloads = Vec::with_capacity(7);
        for i in 0..7usize {
            let payload_str = format!("msg-{}-{}", i, unique_id());
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

        // ── 3. group A fetches all 7 ──────────────────────────────────────────
        let fetch_a = fetch(&client, &mail_address, &group_a, 10).await;
        assert!(
            fetch_a.error.is_empty(),
            "group A fetch error: {}",
            fetch_a.error
        );
        assert_eq!(
            fetch_a.messages.len(),
            7,
            "group A: expected 7 messages, got {}",
            fetch_a.messages.len()
        );
        let received_a: Vec<String> = fetch_a.messages.iter().map(|m| m.payload.clone()).collect();
        println!("group A received: {:?}", received_a);
        for sent in &sent_payloads {
            assert!(
                received_a.iter().any(|r| r == sent),
                "group A missing payload '{}'",
                sent
            );
        }

        // ── 4. group B fetches all 7 independently ────────────────────────────
        let fetch_b = fetch(&client, &mail_address, &group_b, 10).await;
        assert!(
            fetch_b.error.is_empty(),
            "group B fetch error: {}",
            fetch_b.error
        );
        assert_eq!(
            fetch_b.messages.len(),
            7,
            "group B: expected 7 messages, got {}",
            fetch_b.messages.len()
        );
        let received_b: Vec<String> = fetch_b.messages.iter().map(|m| m.payload.clone()).collect();
        println!("group B received: {:?}", received_b);
        for sent in &sent_payloads {
            assert!(
                received_b.iter().any(|r| r == sent),
                "group B missing payload '{}'",
                sent
            );
        }

        // ── 5. group A acks the last message → next fetch yields nothing ──────
        let last_msg_id = fetch_a.messages.last().unwrap().msg_id;
        let ack_reply = ack(&client, &mail_address, &group_a, last_msg_id).await;
        assert!(ack_reply.error.is_empty(), "ack error: {}", ack_reply.error);

        let fetch_a2 = fetch(&client, &mail_address, &group_a, 10).await;
        assert!(
            fetch_a2.error.is_empty(),
            "group A re-fetch error: {}",
            fetch_a2.error
        );
        assert_eq!(
            fetch_a2.messages.len(),
            0,
            "group A: expected 0 messages after ack, got {}",
            fetch_a2.messages.len()
        );

        // ── 6. group B has not acked yet → can still re-fetch same messages ───
        let fetch_b2 = fetch(&client, &mail_address, &group_b, 10).await;
        assert!(
            fetch_b2.error.is_empty(),
            "group B re-fetch error: {}",
            fetch_b2.error
        );
        assert_eq!(
            fetch_b2.messages.len(),
            7,
            "group B: expected 7 messages (not acked), got {}",
            fetch_b2.messages.len()
        );
    }
}
