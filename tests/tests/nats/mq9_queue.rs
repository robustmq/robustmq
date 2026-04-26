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

    use admin_server::cluster::share_group::{
        ShareGroupDetailReq, ShareGroupDetailResp, ShareGroupListReq,
    };
    use admin_server::tool::PageReplyData;
    use async_nats::Client;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use futures::StreamExt;
    use metadata_struct::mq9::Priority;
    use metadata_struct::mqtt::share_group::ShareGroup;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{CreateMailboxReq, Mq9Reply};
    use tokio::time::sleep;

    use crate::nats::common::{admin_client, nats_connect, DEFAULT_TENANT};

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
    async fn test_mq9_queue() {
        let client = nats_connect().await;
        let queue_group = format!("qg-{}", unique_id());

        // ── 1. create mailbox ─────────────────────────────────────────────────
        let req = CreateMailboxReq {
            ttl: None,
            public: false,
            prefix: None,
            name: None,
            desc: "mq9_queue test".to_string(),
        };
        let reply = create_mail(&client, &req).await;
        assert!(!reply.is_error(), "create mail error: {}", reply.error);
        let mail_address = reply.mail_address.unwrap();

        sleep(Duration::from_secs(3)).await;

        // ── 2. publish 7 messages before subscribers exist ────────────────────
        // MQ9 uses Earliest offset strategy, so pre-published messages are
        // delivered after subscribers connect.
        let mut sent_payloads = Vec::with_capacity(7);
        for i in 0..7usize {
            let payload_str = format!("msg-{}-{}", i, unique_id());
            let subject = Mq9Command::MailboxMsg {
                mail_address: mail_address.clone(),
                priority: Priority::Normal,
            }
            .to_subject();
            let reply = nats_request(&client, subject, Bytes::from(payload_str.clone())).await;
            assert!(!reply.is_error(), "pub {}: {}", i, reply.error);
            sent_payloads.push(payload_str);
        }

        // ── 3. subscribe three times with the same queue group ────────────────
        let sub_subject = Mq9Command::MailboxSub {
            mail_address: mail_address.clone(),
        }
        .to_subject();
        let sub1 = client
            .queue_subscribe(sub_subject.clone(), queue_group.clone())
            .await
            .unwrap();
        let sub2 = client
            .queue_subscribe(sub_subject.clone(), queue_group.clone())
            .await
            .unwrap();
        let sub3 = client
            .queue_subscribe(sub_subject.clone(), queue_group.clone())
            .await
            .unwrap();

        // ── 4. wait for push tasks to start ──────────────────────────────────
        sleep(Duration::from_secs(10)).await;

        // ── 5. verify group membership via admin API ──────────────────────────
        let admin = admin_client();
        let list_req = ShareGroupListReq {
            tenant: Some(DEFAULT_TENANT.to_string()),
            group_name: Some(queue_group.clone()),
            ..Default::default()
        };
        let group_list: PageReplyData<Vec<ShareGroup>> =
            admin.get_share_group_list(&list_req).await.unwrap();
        println!("[after sleep] share group list: {:#?}", group_list);
        assert_eq!(
            group_list.data.len(),
            1,
            "[after sleep] expected 1 share group"
        );
        assert_eq!(group_list.data[0].group_name, queue_group);

        let detail_req = ShareGroupDetailReq {
            tenant: DEFAULT_TENANT.to_string(),
            group_name: queue_group.clone(),
        };
        let detail: ShareGroupDetailResp = admin.get_share_group_detail(&detail_req).await.unwrap();
        println!("[after sleep] share group detail: {:#?}", detail);
        assert_eq!(
            detail.members.len(),
            3,
            "[after sleep] expected 3 members, got {}",
            detail.members.len()
        );

        // ── 6. collect messages from each subscriber within a 10s window ──────
        let collect = |mut sub: async_nats::Subscriber| async move {
            let mut msgs = Vec::new();
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            loop {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match tokio::time::timeout(remaining, sub.next()).await {
                    Ok(Some(msg)) => msgs.push(String::from_utf8_lossy(&msg.payload).to_string()),
                    _ => break,
                }
            }
            msgs
        };

        let r1 = collect(sub1).await;
        let r2 = collect(sub2).await;
        let r3 = collect(sub3).await;

        println!("sub1 received ({} msgs): {:?}", r1.len(), r1);
        println!("sub2 received ({} msgs): {:?}", r2.len(), r2);
        println!("sub3 received ({} msgs): {:?}", r3.len(), r3);

        // ── 7. total must be 7, no duplicates ─────────────────────────────────
        let mut all: Vec<_> = r1.iter().chain(r2.iter()).chain(r3.iter()).collect();
        all.sort();
        all.dedup();
        assert_eq!(
            all.len(),
            7,
            "expected 7 unique messages, got {}",
            all.len()
        );

        // ── 8. distribution must be [2, 2, 3] in any order ───────────────────
        let mut counts = [r1.len(), r2.len(), r3.len()];
        counts.sort();
        assert_eq!(
            counts,
            [2, 2, 3],
            "expected distribution [2,2,3], got {:?}",
            counts
        );

        // ── 9. verify share group still exists ───────────────────────────────
        let group_list2: PageReplyData<Vec<ShareGroup>> =
            admin.get_share_group_list(&list_req).await.unwrap();
        println!("share group list (final): {:#?}", group_list2);
        assert_eq!(
            group_list2.data.len(),
            1,
            "expected 1 share group, got {}",
            group_list2.data.len()
        );
        assert_eq!(group_list2.data[0].group_name, queue_group);
        assert_eq!(group_list2.data[0].tenant, DEFAULT_TENANT);
    }
}
