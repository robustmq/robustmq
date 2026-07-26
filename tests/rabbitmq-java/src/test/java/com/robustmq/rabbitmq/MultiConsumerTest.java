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
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.GetResponse;
import org.junit.jupiter.api.Test;

/** Multiple queues and multiple competing consumers on one queue; see MultiNodeTest for cross-node fan-out. */
class MultiConsumerTest {

    @Test
    void multipleQueuesEachDeliverOnlyTheirOwnMessages() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            int queueCount = 3;
            int perQueue = 5;

            List<String> queues = new ArrayList<>();
            for (int i = 0; i < queueCount; i++) {
                queues.add(Support.declareQueue(channel, "it-multi-q" + i));
            }

            Map<String, List<String>> receivedByQueue = new ConcurrentHashMap<>();
            Map<String, CountDownLatch> latches = new ConcurrentHashMap<>();
            for (String queue : queues) {
                receivedByQueue.put(queue, new CopyOnWriteArrayList<>());
                latches.put(queue, new CountDownLatch(perQueue));
                channel.basicConsume(queue, true, (tag, delivery) -> {
                    receivedByQueue.get(queue).add(new String(delivery.getBody(), StandardCharsets.UTF_8));
                    latches.get(queue).countDown();
                }, tag -> {});
            }

            for (String queue : queues) {
                for (int i = 0; i < perQueue; i++) {
                    channel.basicPublish("", queue, null, (queue + "-" + i).getBytes(StandardCharsets.UTF_8));
                }
            }

            for (String queue : queues) {
                assertTrue(latches.get(queue).await(15, TimeUnit.SECONDS));
                List<String> received = receivedByQueue.get(queue);
                assertEquals(perQueue, received.size());
                for (String body : received) {
                    assertTrue(body.startsWith(queue + "-"), "wrong queue's message: " + body);
                }
            }
        }
    }

    @Test
    void twoCompetingConsumersOnOneQueueSplitTheMessagesRoundRobin() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-competing");

            int total = 20;
            CountDownLatch allReceived = new CountDownLatch(total);
            AtomicInteger countA = new AtomicInteger();
            AtomicInteger countB = new AtomicInteger();
            channel.basicConsume(queue, true, (tag, d) -> {
                countA.incrementAndGet();
                allReceived.countDown();
            }, tag -> {});
            channel.basicConsume(queue, true, (tag, d) -> {
                countB.incrementAndGet();
                allReceived.countDown();
            }, tag -> {});

            for (int i = 0; i < total; i++) {
                channel.basicPublish("", queue, null, ("v" + i).getBytes(StandardCharsets.UTF_8));
            }

            assertTrue(allReceived.await(15, TimeUnit.SECONDS));
            assertEquals(total, countA.get() + countB.get(), "no message may be duplicated or dropped");
            assertTrue(countA.get() > 0 && countB.get() > 0, "both consumers should get a share");
        }
    }

    @Test
    void cancelledConsumerStopsReceivingFurtherMessages() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-cancel");

            AtomicInteger received = new AtomicInteger();
            String consumerTag =
                    channel.basicConsume(queue, true, (tag, d) -> received.incrementAndGet(), tag -> {});

            channel.basicPublish("", queue, null, "before-cancel".getBytes(StandardCharsets.UTF_8));
            Support.awaitCount(received, 1, 10);

            channel.basicCancel(consumerTag);
            Thread.sleep(300); // let the cancel land before publishing more
            channel.basicPublish("", queue, null, "after-cancel".getBytes(StandardCharsets.UTF_8));
            Thread.sleep(500);

            assertEquals(1, received.get());
            GetResponse resp = Support.pollGet(channel, queue, true);
            assertEquals("after-cancel", new String(resp.getBody(), StandardCharsets.UTF_8));
        }
    }
}
