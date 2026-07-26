/*
 * Copyright 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.robustmq.rabbitmq;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;

import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.GetResponse;
import org.junit.jupiter.api.Test;

/** Java-client counterpart of the Rust basic_get_test.rs (lapin) suite, same scenarios. */
class BasicGetTest {

    @Test
    void basicGetReturnsPublishedMessage() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-get");
            channel.basicPublish("", queue, null, "payload".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, false);
            assertNotNull(resp);
            assertEquals("payload", new String(resp.getBody(), StandardCharsets.UTF_8));
            channel.basicAck(resp.getEnvelope().getDeliveryTag(), false);
        }
    }

    @Test
    void basicGetOnEmptyQueueReturnsNull() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-get-empty");
            assertNull(channel.basicGet(queue, true));
        }
    }

    @Test
    void basicAckRemovesMessageFromQueue() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-ack");
            channel.basicPublish("", queue, null, "once".getBytes(StandardCharsets.UTF_8));

            GetResponse first = Support.pollGet(channel, queue, false);
            assertNotNull(first);
            channel.basicAck(first.getEnvelope().getDeliveryTag(), false);

            assertNull(channel.basicGet(queue, true));
        }
    }

    @Test
    void basicNackWithRequeueRedeliversMessage() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-nack");
            channel.basicPublish("", queue, null, "retry-me".getBytes(StandardCharsets.UTF_8));

            GetResponse first = Support.pollGet(channel, queue, false);
            assertNotNull(first);
            channel.basicNack(first.getEnvelope().getDeliveryTag(), false, true);

            GetResponse redelivered = Support.pollGet(channel, queue, true);
            assertNotNull(redelivered);
            assertEquals("retry-me", new String(redelivered.getBody(), StandardCharsets.UTF_8));
            assertTrue(redelivered.getEnvelope().isRedeliver());
        }
    }

    @Test
    void noAckTrueConsumesImmediately() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-noack");
            channel.basicPublish("", queue, null, "fire-and-forget".getBytes(StandardCharsets.UTF_8));

            assertNotNull(Support.pollGet(channel, queue, true));
            assertNull(channel.basicGet(queue, true));
        }
    }

    @Test
    void deliveryTagIncrementsPerChannel() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-tag");
            channel.basicPublish("", queue, null, "a".getBytes(StandardCharsets.UTF_8));
            channel.basicPublish("", queue, null, "b".getBytes(StandardCharsets.UTF_8));

            GetResponse first = Support.pollGet(channel, queue, false);
            GetResponse second = Support.pollGet(channel, queue, false);
            assertNotNull(first);
            assertNotNull(second);
            assertTrue(second.getEnvelope().getDeliveryTag() > first.getEnvelope().getDeliveryTag());
            channel.basicAck(first.getEnvelope().getDeliveryTag(), false);
            channel.basicAck(second.getEnvelope().getDeliveryTag(), false);
        }
    }
}
