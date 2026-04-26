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
    use admin_server::cluster::message::ReadMessageReq;
    use admin_server::nats::mail::MailListReq;
    use async_nats::Client;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use metadata_struct::mq9::mail::MQ9Mail;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{CreateMailboxReq, Mq9Reply};
    use mq9_core::public::{StoragePublicData, MQ9_SYSTEM_PUBLIC_MAIL};
    use std::time::Duration;
    use tokio::time::sleep;

    use crate::nats::common::{nats_connect, DEFAULT_TENANT};

    const TTL: u64 = 30;
    // GC runs every 60s; wait TTL + one full GC interval to be sure
    const WAIT_AFTER_TTL: u64 = TTL + 65;

    async fn create_mail(client: &Client, req: &CreateMailboxReq) -> Mq9Reply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        let subject = Mq9Command::MailboxCreate.to_subject();
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<Mq9Reply>(&msg.payload).unwrap()
    }

    #[tokio::test]
    async fn mq9_mail_test() {
        let admin_client = create_test_env().await;
        let nats_client = nats_connect().await;

        // ── create private mail (ttl=30) ──────────────────────────────────────
        let req = CreateMailboxReq {
            ttl: Some(TTL),
            public: false,
            name: None,
            prefix: None,
            desc: "test private mail".to_string(),
        };
        let reply = create_mail(&nats_client, &req).await;
        println!("create private mail reply: {:?}", reply);

        assert!(!reply.is_error(), "unexpected error: {}", reply.error);
        assert!(
            !reply.mail_address.as_deref().unwrap_or("").is_empty(),
            "mail_address should not be empty"
        );
        assert!(reply.is_new.unwrap_or(false), "should be a new mail");
        let mail_address = reply.mail_address.unwrap();

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
        assert!(!mail.public, "should be private");
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
    async fn mq9_public_mail_test() {
        let admin_client = create_test_env().await;
        let nats_client = nats_connect().await;

        // ── create public mail (ttl=30, named) ────────────────────────────────
        let public_name = format!("pub-mail-{}", unique_id());
        let req = CreateMailboxReq {
            ttl: Some(TTL),
            public: true,
            prefix: None,
            name: Some(public_name.clone()),
            desc: "test public mail".to_string(),
        };
        let reply = create_mail(&nats_client, &req).await;
        println!("create public mail reply: {:?}", reply);

        assert!(!reply.is_error(), "unexpected error: {}", reply.error);
        assert_eq!(
            reply.mail_address.as_deref().unwrap_or(""),
            public_name,
            "public mail_address should equal the provided name"
        );
        assert!(reply.is_new.unwrap_or(false), "should be a new mail");

        // ── list mail via admin — verify mail exists with correct flags ────────
        let list_req = MailListReq {
            mail_address: Some(public_name.clone()),
            ..Default::default()
        };
        let mail_list = admin_client
            .get_mail_list::<_, Vec<MQ9Mail>>(&list_req)
            .await
            .unwrap();
        println!("public mail list after create: {:#?}", mail_list);

        assert_eq!(mail_list.data.len(), 1, "expected exactly 1 public mail");
        let mail = &mail_list.data[0];
        assert_eq!(mail.mail_address, public_name);
        assert!(mail.public, "should be public");
        assert_eq!(mail.ttl, TTL);
        assert!(mail.create_time > 0);

        // ── read System topic — verify public mail entry exists ──────────────
        let read_req = ReadMessageReq {
            tenant: DEFAULT_TENANT.to_string(),
            topic: MQ9_SYSTEM_PUBLIC_MAIL.to_string(),
            offset: 0,
        };
        let read_resp = admin_client
            .read_message::<_, admin_server::cluster::message::ReadMessageResp>(&read_req)
            .await
            .unwrap();
        println!("$SYSTEM.PUBLIC messages: {:#?}", read_resp.messages);

        let found = read_resp.messages.iter().any(|row| {
            serde_json::from_str::<StoragePublicData>(&row.content)
                .map(|d| d.mail_address == public_name)
                .unwrap_or(false)
        });
        assert!(
            found,
            "public mail '{}' should appear in $SYSTEM.PUBLIC topic",
            public_name
        );

        // ── wait for TTL + GC cycle ────────────────────────────────────────────
        println!("waiting {}s for TTL expiry and GC...", WAIT_AFTER_TTL);
        sleep(Duration::from_secs(WAIT_AFTER_TTL)).await;

        // ── list mail — verify mail is gone ───────────────────────────────────
        let mail_list_after = admin_client
            .get_mail_list::<_, Vec<MQ9Mail>>(&list_req)
            .await
            .unwrap();
        println!("public mail list after TTL expiry: {:#?}", mail_list_after);
        assert_eq!(
            mail_list_after.data.len(),
            0,
            "public mail should be removed after TTL expiry"
        );
    }

    #[tokio::test]
    async fn mq9_mail_prefix_test() {
        let nats_client = nats_connect().await;

        // ── create mail with prefix ───────────────────────────────────────────
        let prefix = format!("risk.{}", &unique_id().to_lowercase()[..8]);
        let req = CreateMailboxReq {
            ttl: None,
            public: false,
            name: None,
            prefix: Some(prefix.clone()),
            desc: "prefix test mail".to_string(),
        };
        let reply = create_mail(&nats_client, &req).await;
        println!("create prefix mail reply: {:?}", reply);

        assert!(!reply.is_error(), "unexpected error: {}", reply.error);
        let mail_address = reply.mail_address.as_deref().unwrap_or("");
        assert!(
            mail_address.starts_with(&prefix),
            "mail_address '{}' should start with prefix '{}'",
            mail_address,
            prefix
        );
        assert!(reply.is_new.unwrap_or(false), "should be a new mailbox");
    }
}
