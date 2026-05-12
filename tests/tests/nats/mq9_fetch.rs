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
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
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

    async fn create_mailbox(client: &Client) -> String {
        let req = MailboxCreateReq {
            name: Some(format!("test{}", unique_id().to_lowercase())),
            ttl: None,
            desc: None,
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let reply: MailboxCreateReply =
            request(client, Mq9Command::MailboxCreate.to_subject(), payload).await;
        assert!(reply.error.is_empty(), "create mail error: {}", reply.error);
        sleep(Duration::from_secs(3)).await;
        reply.mail_address
    }

    async fn send_msg(client: &Client, mail_address: &str, payload: &str) -> MsgSendReply {
        let subject = Mq9Command::MsgSend {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        let reply: MsgSendReply = request(client, subject, Bytes::from(payload.to_string())).await;
        assert!(reply.error.is_empty(), "send error: {}", reply.error);
        reply
    }

    async fn fetch_with(client: &Client, mail_address: &str, req: MsgFetchReq) -> MsgFetchReply {
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let subject = Mq9Command::MsgFetch {
            mail_address: mail_address.to_string(),
        }
        .to_subject();
        let reply: MsgFetchReply = request(client, subject, payload).await;
        assert!(reply.error.is_empty(), "fetch error: {}", reply.error);
        reply
    }

    async fn fetch_earliest(
        client: &Client,
        mail_address: &str,
        group_name: &str,
        num_msgs: Option<u32>,
    ) -> MsgFetchReply {
        fetch_with(
            client,
            mail_address,
            MsgFetchReq {
                group_name: Some(group_name.to_string()),
                deliver: DeliverPolicy::Earliest,
                from_time: None,
                from_id: None,
                force_deliver: None,
                config: num_msgs.map(|n| MsgFetchConfig {
                    num_msgs: Some(n),
                    max_wait_ms: None,
                }),
            },
        )
        .await
    }

    async fn ack(client: &Client, mail_address: &str, group_name: &str, msg_id: u64) {
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
        let reply: MsgAckReply = request(client, subject, payload).await;
        assert!(reply.error.is_empty(), "ack error: {}", reply.error);
    }

    #[tokio::test]
    async fn test_fetch_earliest() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
        }

        let reply = fetch_earliest(&client, &mail_address, &group, None).await;
        assert_eq!(
            reply.messages.len(),
            5,
            "expected 5, got {}",
            reply.messages.len()
        );
        for (i, msg) in reply.messages.iter().enumerate() {
            assert_eq!(
                msg.payload,
                format!("msg-{}", i),
                "order mismatch at index {}",
                i
            );
        }

        let last_id = reply.messages.last().unwrap().msg_id;
        ack(&client, &mail_address, &group, last_id).await;

        let reply = fetch_earliest(&client, &mail_address, &group, None).await;
        assert_eq!(
            reply.messages.len(),
            0,
            "expected 0 after ack, got {}",
            reply.messages.len()
        );
    }

    #[tokio::test]
    async fn test_fetch_latest() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
        }

        let reply = fetch_with(
            &client,
            &mail_address,
            MsgFetchReq {
                group_name: Some(group.clone()),
                deliver: DeliverPolicy::Latest,
                from_time: None,
                from_id: None,
                force_deliver: None,
                config: None,
            },
        )
        .await;
        assert_eq!(
            reply.messages.len(),
            1,
            "latest: expected 1, got {}",
            reply.messages.len()
        );
        assert_eq!(reply.messages[0].payload, "msg-4", "latest: wrong payload");
    }

    #[tokio::test]
    async fn test_fetch_from_time() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("early-{}", i)).await;
        }

        sleep(Duration::from_secs(2)).await;
        let mid_ts = now_second();
        sleep(Duration::from_secs(1)).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("late-{}", i)).await;
        }

        let reply = fetch_with(
            &client,
            &mail_address,
            MsgFetchReq {
                group_name: Some(group.clone()),
                deliver: DeliverPolicy::FromTime,
                from_time: Some(mid_ts),
                from_id: None,
                force_deliver: None,
                config: None,
            },
        )
        .await;
        assert_eq!(
            reply.messages.len(),
            5,
            "from_time: expected 5, got {}",
            reply.messages.len()
        );
        for msg in &reply.messages {
            assert!(
                msg.payload.starts_with("late-"),
                "from_time: unexpected '{}'",
                msg.payload
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_from_id() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        let mut msg_ids = Vec::new();
        for i in 0..10u32 {
            let r = send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
            msg_ids.push(r.msg_id as u64);
        }

        let from_id = msg_ids[5];
        let reply = fetch_with(
            &client,
            &mail_address,
            MsgFetchReq {
                group_name: Some(group.clone()),
                deliver: DeliverPolicy::FromId,
                from_time: None,
                from_id: Some(from_id),
                force_deliver: None,
                config: None,
            },
        )
        .await;
        assert_eq!(
            reply.messages.len(),
            5,
            "from_id: expected 5, got {}",
            reply.messages.len()
        );
        for msg in &reply.messages {
            assert!(
                msg.msg_id >= from_id,
                "from_id: msg_id {} is before from_id {}",
                msg.msg_id,
                from_id
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_num_msgs_limit() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..9u32 {
            send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
        }

        let page1 = fetch_earliest(&client, &mail_address, &group, Some(3)).await;
        assert_eq!(
            page1.messages.len(),
            3,
            "page1: expected 3, got {}",
            page1.messages.len()
        );
        ack(
            &client,
            &mail_address,
            &group,
            page1.messages.last().unwrap().msg_id,
        )
        .await;

        let page2 = fetch_earliest(&client, &mail_address, &group, Some(3)).await;
        assert_eq!(
            page2.messages.len(),
            3,
            "page2: expected 3, got {}",
            page2.messages.len()
        );
        ack(
            &client,
            &mail_address,
            &group,
            page2.messages.last().unwrap().msg_id,
        )
        .await;

        let page3 = fetch_earliest(&client, &mail_address, &group, Some(3)).await;
        assert_eq!(
            page3.messages.len(),
            3,
            "page3: expected 3, got {}",
            page3.messages.len()
        );
        ack(
            &client,
            &mail_address,
            &group,
            page3.messages.last().unwrap().msg_id,
        )
        .await;

        let done = fetch_earliest(&client, &mail_address, &group, Some(3)).await;
        assert_eq!(
            done.messages.len(),
            0,
            "done: expected 0, got {}",
            done.messages.len()
        );
    }

    #[tokio::test]
    async fn test_fetch_two_groups_independent() {
        let client = nats_connect().await;
        let group_a = format!("grp-a-{}", unique_id());
        let group_b = format!("grp-b-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
        }

        let ra = fetch_earliest(&client, &mail_address, &group_a, None).await;
        let rb = fetch_earliest(&client, &mail_address, &group_b, None).await;
        assert_eq!(ra.messages.len(), 5, "group-a: expected 5");
        assert_eq!(rb.messages.len(), 5, "group-b: expected 5");

        ack(
            &client,
            &mail_address,
            &group_a,
            ra.messages.last().unwrap().msg_id,
        )
        .await;

        let ra2 = fetch_earliest(&client, &mail_address, &group_a, None).await;
        assert_eq!(
            ra2.messages.len(),
            0,
            "group-a after ack: expected 0, got {}",
            ra2.messages.len()
        );

        let rb2 = fetch_earliest(&client, &mail_address, &group_b, None).await;
        assert_eq!(
            rb2.messages.len(),
            5,
            "group-b should still see 5, got {}",
            rb2.messages.len()
        );
    }

    #[tokio::test]
    async fn test_fetch_force_deliver() {
        let client = nats_connect().await;
        let group = format!("grp-{}", unique_id());
        let mail_address = create_mailbox(&client).await;

        for i in 0..5u32 {
            send_msg(&client, &mail_address, &format!("msg-{}", i)).await;
        }

        let reply = fetch_earliest(&client, &mail_address, &group, None).await;
        assert_eq!(reply.messages.len(), 5);
        ack(
            &client,
            &mail_address,
            &group,
            reply.messages.last().unwrap().msg_id,
        )
        .await;

        let empty = fetch_earliest(&client, &mail_address, &group, None).await;
        assert_eq!(
            empty.messages.len(),
            0,
            "expected 0 after ack, got {}",
            empty.messages.len()
        );

        let forced = fetch_with(
            &client,
            &mail_address,
            MsgFetchReq {
                group_name: Some(group.clone()),
                deliver: DeliverPolicy::Earliest,
                from_time: None,
                from_id: None,
                force_deliver: Some(true),
                config: None,
            },
        )
        .await;
        assert_eq!(
            forced.messages.len(),
            5,
            "force_deliver: expected 5, got {}",
            forced.messages.len()
        );
    }
}
