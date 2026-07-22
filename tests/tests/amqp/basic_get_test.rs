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
    use crate::amqp::common::{
        amqp_channel, amqp_connection, declare_test_queue, unique_queue_name,
    };
    use lapin::options::{BasicAckOptions, BasicGetOptions, BasicNackOptions, BasicPublishOptions};
    use lapin::BasicProperties;

    #[tokio::test]
    async fn basic_get_empty_queue_returns_none() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_get_empty");
        declare_test_queue(&channel, &queue).await;

        let msg = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call");

        assert!(msg.is_none());
    }

    #[tokio::test]
    async fn basic_get_returns_published_message() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_get_publish");
        declare_test_queue(&channel, &queue).await;

        channel
            .basic_publish(
                "",
                &queue,
                BasicPublishOptions::default(),
                b"hello-amqp",
                BasicProperties::default(),
            )
            .await
            .expect("publish")
            .await
            .expect("publisher confirm");

        let msg = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("message should be present");

        assert_eq!(msg.delivery.data, b"hello-amqp");
        assert_eq!(msg.delivery.delivery_tag, 1);
        assert!(!msg.delivery.redelivered);
    }

    #[tokio::test]
    async fn basic_get_delivery_tag_increments_per_channel() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_get_tags");
        declare_test_queue(&channel, &queue).await;

        for body in [b"one".as_slice(), b"two".as_slice()] {
            channel
                .basic_publish(
                    "",
                    &queue,
                    BasicPublishOptions::default(),
                    body,
                    BasicProperties::default(),
                )
                .await
                .expect("publish")
                .await
                .expect("publisher confirm");
        }

        let first = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("first message present");
        let second = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("second message present");

        assert_eq!(first.delivery.delivery_tag, 1);
        assert_eq!(second.delivery.delivery_tag, 2);
        assert_eq!(first.delivery.data, b"one");
        assert_eq!(second.delivery.data, b"two");
    }

    #[tokio::test]
    async fn basic_ack_removes_message_from_queue() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_ack");
        declare_test_queue(&channel, &queue).await;

        channel
            .basic_publish(
                "",
                &queue,
                BasicPublishOptions::default(),
                b"ack-me",
                BasicProperties::default(),
            )
            .await
            .expect("publish")
            .await
            .expect("publisher confirm");

        let msg = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("message present");

        channel
            .basic_ack(msg.delivery.delivery_tag, BasicAckOptions::default())
            .await
            .expect("ack");

        let after_ack = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call");
        assert!(after_ack.is_none());
    }

    #[tokio::test]
    async fn basic_get_no_ack_true_consumes_immediately() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_get_no_ack");
        declare_test_queue(&channel, &queue).await;

        channel
            .basic_publish(
                "",
                &queue,
                BasicPublishOptions::default(),
                b"no-ack",
                BasicProperties::default(),
            )
            .await
            .expect("publish")
            .await
            .expect("publisher confirm");

        let msg = channel
            .basic_get(&queue, BasicGetOptions { no_ack: true })
            .await
            .expect("basic_get call")
            .expect("message present");
        assert_eq!(msg.delivery.data, b"no-ack");

        let after = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call");
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn basic_nack_requeue_true_redelivers_message() {
        let conn = amqp_connection().await;
        let channel = amqp_channel(&conn).await;
        let queue = unique_queue_name("basic_nack_requeue");
        declare_test_queue(&channel, &queue).await;

        channel
            .basic_publish(
                "",
                &queue,
                BasicPublishOptions::default(),
                b"requeue-me",
                BasicProperties::default(),
            )
            .await
            .expect("publish")
            .await
            .expect("publisher confirm");

        let first = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("message present");
        assert!(!first.delivery.redelivered);

        channel
            .basic_nack(
                first.delivery.delivery_tag,
                BasicNackOptions {
                    multiple: false,
                    requeue: true,
                },
            )
            .await
            .expect("nack");

        let second = channel
            .basic_get(&queue, BasicGetOptions { no_ack: false })
            .await
            .expect("basic_get call")
            .expect("message should be redelivered");

        assert_eq!(second.delivery.data, b"requeue-me");
        assert!(second.delivery.redelivered);

        channel
            .basic_ack(second.delivery.delivery_tag, BasicAckOptions::default())
            .await
            .expect("ack");
    }
}
