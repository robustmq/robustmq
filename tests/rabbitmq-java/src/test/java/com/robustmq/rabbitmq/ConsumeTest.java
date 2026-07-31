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
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

import com.rabbitmq.client.AMQP;
import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.DeliverCallback;
import com.rabbitmq.client.GetResponse;
import org.junit.jupiter.api.Test;

/** Single-queue, single-consumer Basic.Consume; see MultiConsumerTest and MultiNodeTest for fan-out. */
class ConsumeTest {

    @Test
    void deliversMessagesInOrder() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-consume-order");

            int total = 20;
            CountDownLatch received = new CountDownLatch(total);
            List<String> bodies = new CopyOnWriteArrayList<>();
            DeliverCallback onDeliver = (consumerTag, delivery) -> {
                bodies.add(new String(delivery.getBody(), StandardCharsets.UTF_8));
                channel.basicAck(delivery.getEnvelope().getDeliveryTag(), false);
                received.countDown();
            };
            channel.basicConsume(queue, false, onDeliver, tag -> {});

            for (int i = 0; i < total; i++) {
                channel.basicPublish("", queue, null, ("v" + i).getBytes(StandardCharsets.UTF_8));
            }

            assertTrue(received.await(10, TimeUnit.SECONDS));
            assertEquals(total, bodies.size());
            for (int i = 0; i < total; i++) {
                assertEquals("v" + i, bodies.get(i));
            }
        }
    }

    @Test
    void noAckDeliversWithoutWaitingForAck() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-consume-noack");

            CountDownLatch received = new CountDownLatch(1);
            channel.basicConsume(queue, true, (tag, delivery) -> received.countDown(), tag -> {});

            channel.basicPublish("", queue, null, "fire-and-forget".getBytes(StandardCharsets.UTF_8));

            assertTrue(received.await(10, TimeUnit.SECONDS));
        }
    }

    @Test
    void getAndConsumeShareOneCursor() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-get-consume-shared");

            channel.basicPublish("", queue, null, "for-get".getBytes(StandardCharsets.UTF_8));
            GetResponse got = Support.pollGet(channel, queue, true);
            assertEquals("for-get", new String(got.getBody(), StandardCharsets.UTF_8));

            CountDownLatch received = new CountDownLatch(1);
            AtomicReference<String> body = new AtomicReference<>();
            channel.basicConsume(queue, true, (tag, delivery) -> {
                body.set(new String(delivery.getBody(), StandardCharsets.UTF_8));
                received.countDown();
            }, tag -> {});

            channel.basicPublish("", queue, null, "for-consume".getBytes(StandardCharsets.UTF_8));

            assertTrue(received.await(10, TimeUnit.SECONDS));
            assertEquals("for-consume", body.get(), "consumer must not re-receive the Get'd message");
        }
    }

    @Test
    void contentPropertiesRoundTripThroughConsume() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-consume-props");

            CountDownLatch received = new CountDownLatch(1);
            AtomicReference<AMQP.BasicProperties> propsRef = new AtomicReference<>();
            channel.basicConsume(queue, true, (tag, delivery) -> {
                propsRef.set(delivery.getProperties());
                received.countDown();
            }, tag -> {});

            AMQP.BasicProperties props = new AMQP.BasicProperties.Builder().contentType("text/plain").build();
            channel.basicPublish("", queue, props, "typed".getBytes(StandardCharsets.UTF_8));

            assertTrue(received.await(10, TimeUnit.SECONDS));
            assertEquals("text/plain", propsRef.get().getContentType());
        }
    }
}
