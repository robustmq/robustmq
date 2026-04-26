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

    use admin_server::cluster::topic::TopicListReq;
    use bytes::Bytes;
    use common_base::uuid::unique_id;
    use futures::StreamExt;
    use metadata_struct::topic::Topic;
    use tokio::time::sleep;

    use crate::nats::common::{admin_client, nats_connect, DEFAULT_TENANT};

    #[tokio::test]
    async fn test_nats_core() {
        let client = nats_connect().await;
        let subject = format!("test.{}", unique_id());
        let payload = format!("hello-{}", unique_id());

        // subscribe before publishing so no messages are missed
        let mut sub = client.subscribe(subject.clone()).await.unwrap();

        client
            .publish(subject.clone(), Bytes::from(payload.clone()))
            .await
            .unwrap();

        // verify the subscriber receives the exact payload
        let received = tokio::time::timeout(Duration::from_secs(5), sub.next())
            .await
            .expect("timeout waiting for message")
            .expect("subscription closed");

        let received_payload = String::from_utf8_lossy(&received.payload).to_string();
        assert_eq!(
            received_payload, payload,
            "received payload mismatch: expected '{}', got '{}'",
            payload, received_payload
        );

        // wait for the broker to register the topic asynchronously
        sleep(Duration::from_secs(3)).await;

        // verify the subject was persisted as a topic in the cluster
        let admin = admin_client();
        let list_req = TopicListReq {
            tenant: Some(DEFAULT_TENANT.to_string()),
            topic_name: Some(subject.clone()),
            ..Default::default()
        };
        let topic_list = admin
            .get_topic_list::<_, Vec<Topic>>(&list_req)
            .await
            .unwrap();

        assert_eq!(
            topic_list.data.len(),
            1,
            "expected 1 topic with name '{}', got {}",
            subject,
            topic_list.data.len()
        );
        assert_eq!(
            topic_list.data[0].topic_name, subject,
            "topic name should equal subject"
        );
    }
}
