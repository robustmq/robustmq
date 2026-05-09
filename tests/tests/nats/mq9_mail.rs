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
    use crate::mqtt::protocol::common::create_test_env;
    use crate::nats::common::nats_connect;
    use admin_server::mq9::mail::MailListReq;
    use async_nats::Client;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use metadata_struct::mq9::mail::MQ9Mail;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{MailboxCreateReply, MailboxCreateReq};
    use std::time::Duration;
    use tokio::time::sleep;

    const TTL: u64 = 30;
    // GC runs every 60s; wait TTL + one full GC interval to be sure
    const WAIT_AFTER_TTL: u64 = TTL + 65;

    async fn create_mail(client: &Client, req: &MailboxCreateReq) -> MailboxCreateReply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        let subject = Mq9Command::MailboxCreate.to_subject();
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<MailboxCreateReply>(&msg.payload).unwrap()
    }

    #[tokio::test]
    async fn mq9_mail_test() {
        let admin_client = create_test_env().await;
        let nats_client = nats_connect().await;

        // ── create private mail (ttl=30) ──────────────────────────────────────
        let req = MailboxCreateReq {
            name: Some(unique_id().to_lowercase().to_string()),
            ttl: Some(TTL),
            desc: None,
        };
        let reply = create_mail(&nats_client, &req).await;
        println!("create private mail reply: {:?}", reply);

        assert!(reply.error.is_empty(), "unexpected error: {}", reply.error);
        assert!(
            !reply.mail_address.is_empty(),
            "mail_address should not be empty"
        );
        let mail_address = reply.mail_address;

        // ── list mail via admin — verify mail exists ───────────────────────────
        let list_req = MailListReq {
            mail_address: Some(mail_address.clone()),
            ..Default::default()
        };
        let mail_list = admin_client
            .get_mail_list::<_, Vec<MQ9Mail>>(&list_req)
            .await
            .unwrap();
        println!("mail list after create: {:#?}", mail_list);

        assert_eq!(mail_list.data.len(), 1, "expected exactly 1 mail");
        let mail = &mail_list.data[0];
        assert_eq!(mail.mail_address, mail_address);
        assert_eq!(mail.ttl, TTL);
        assert!(mail.create_time > 0);

        // ── wait for TTL + GC cycle ────────────────────────────────────────────
        println!("waiting {}s for TTL expiry and GC...", WAIT_AFTER_TTL);
        sleep(Duration::from_secs(WAIT_AFTER_TTL)).await;

        // ── list mail — verify mail is gone ───────────────────────────────────
        let mail_list_after = admin_client
            .get_mail_list::<_, Vec<MQ9Mail>>(&list_req)
            .await
            .unwrap();
        println!("mail list after TTL expiry: {:#?}", mail_list_after);
        assert_eq!(
            mail_list_after.data.len(),
            0,
            "mail should be removed after TTL expiry"
        );
    }

    #[tokio::test]
    async fn mq9_mail_name_test() {
        let nats_client = nats_connect().await;

        // ── create mail with prefix ───────────────────────────────────────────
        let name = format!("risk{}", unique_id().to_lowercase());
        let req = MailboxCreateReq {
            name: Some(name.clone()),
            ttl: None,
            desc: None,
        };
        let reply = create_mail(&nats_client, &req).await;
        println!("create prefix mail reply: {:?}", reply);

        assert!(reply.error.is_empty(), "unexpected error: {}", reply.error);
        let mail_address = &reply.mail_address;
        assert!(
            mail_address.starts_with(&name),
            "mail_address '{}' should start with prefix '{}'",
            mail_address,
            name
        );
    }
}
