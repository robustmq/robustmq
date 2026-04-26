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
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use futures::StreamExt;
    use metadata_struct::mqtt::share_group::ShareGroup;
    use tokio::time::sleep;

    use crate::nats::common::{admin_client, nats_connect, DEFAULT_TENANT};

    #[tokio::test]
    async fn test_nats_queue() {
        let client = nats_connect().await;
        let subject = format!("test.queue.{}", unique_id());
        let queue_group = format!("qg-{}", unique_id());

        // publish one warm-up message to create the topic/shard before any subscriber exists
        client
            .publish(subject.clone(), Bytes::from("warmup"))
            .await
            .unwrap();
        client.flush().await.unwrap();

        // subscribe three times with the same queue group — each gets a distinct sid
        let sub1 = client
            .queue_subscribe(subject.clone(), queue_group.clone())
            .await
            .unwrap();
        let sub2 = client
            .queue_subscribe(subject.clone(), queue_group.clone())
            .await
            .unwrap();
        let sub3 = client
            .queue_subscribe(subject.clone(), queue_group.clone())
            .await
            .unwrap();

        // wait for all three subscribers to be registered and push tasks to start
        sleep(Duration::from_secs(10)).await;

        // print share group and detail after sleep to verify subscriber state
        let admin = admin_client();
        let list_req = ShareGroupListReq {
            tenant: Some(DEFAULT_TENANT.to_string()),
            group_name: Some(queue_group.clone()),
            ..Default::default()
        };
        let group_list: PageReplyData<Vec<ShareGroup>> =
            admin.get_share_group_list(&list_req).await.unwrap();
        println!("[after sleep] share group list: {:#?}", group_list);

        let detail_req = ShareGroupDetailReq {
            tenant: DEFAULT_TENANT.to_string(),
            group_name: queue_group.clone(),
        };
        let detail: ShareGroupDetailResp =
            admin.get_share_group_detail(&detail_req).await.unwrap();
        println!("[after sleep] share group detail: {:#?}", detail);

        // publish 6 messages after subscribers are stable
        for i in 0..6u32 {
            client
                .publish(subject.clone(), Bytes::from(format!("msg-{}", i)))
                .await
                .unwrap();
        }
        client.flush().await.unwrap();

        // collect received messages from each subscriber within a 5s window
        let collect = |mut sub: async_nats::Subscriber| async move {
            let mut msgs = Vec::new();
            let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
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

        // total must be 7 (1 warmup + 6 actual), no duplicates
        let mut all: Vec<_> = r1.iter().chain(r2.iter()).chain(r3.iter()).collect();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 7, "expected 7 unique messages, got {}", all.len());

        // each subscriber should receive at least 1 message (roughly balanced: 2~3 each)
        assert!(r1.len() >= 1, "sub1 received nothing");
        assert!(r2.len() >= 1, "sub2 received nothing");
        assert!(r3.len() >= 1, "sub3 received nothing");

        // verify share group exists via admin API
        let admin = admin_client();
        let list_req = ShareGroupListReq {
            tenant: Some(DEFAULT_TENANT.to_string()),
            group_name: Some(queue_group.clone()),
            ..Default::default()
        };
        let group_list: PageReplyData<Vec<ShareGroup>> =
            admin.get_share_group_list(&list_req).await.unwrap();
        println!("share group list: {:#?}", group_list);

        assert_eq!(
            group_list.data.len(),
            1,
            "expected 1 share group, got {}",
            group_list.data.len()
        );
        assert_eq!(group_list.data[0].group_name, queue_group);
        assert_eq!(group_list.data[0].tenant, DEFAULT_TENANT);

        // verify share group detail: 3 members
        let detail_req = ShareGroupDetailReq {
            tenant: DEFAULT_TENANT.to_string(),
            group_name: queue_group.clone(),
        };
        let detail: ShareGroupDetailResp =
            admin.get_share_group_detail(&detail_req).await.unwrap();
        println!("share group detail: {:#?}", detail);

        assert_eq!(
            detail.members.len(),
            3,
            "expected 3 members, got {}",
            detail.members.len()
        );
        for member in &detail.members {
            assert_eq!(member.tenant, DEFAULT_TENANT);
            assert_eq!(member.group_name, queue_group);
            assert_eq!(member.sub_path, subject);
        }
    }
}
