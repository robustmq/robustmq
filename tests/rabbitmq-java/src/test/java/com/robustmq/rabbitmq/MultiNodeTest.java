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
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.ConnectionFactory;
import com.rabbitmq.client.GetResponse;
import org.junit.jupiter.api.Test;

/**
 * Runs against a real 3-node cluster (scripts/cluster.sh, ports
 * 5672/5772/5872 by default; override with -Damqp.node{1,2,3}Port).
 * Basic.Get/Consume must work correctly regardless of which node a
 * connection lands on relative to the queue's elected leader.
 */
class MultiNodeTest {

    private static int nodePort(int n, int fallback) {
        String v = System.getProperty("amqp.node" + n + "Port", System.getenv("AMQP_NODE" + n + "_PORT"));
        return v != null ? Integer.parseInt(v) : fallback;
    }

    private static Connection connectToNode(int n, int fallbackPort) throws Exception {
        ConnectionFactory factory = Support.newFactory();
        factory.setPort(nodePort(n, fallbackPort));
        return factory.newConnection();
    }

    @Test
    void publishOnOneNodeGetOnAnotherNodeSeesTheMessage() throws Exception {
        try (Connection nodeA = connectToNode(1, 5672);
                Connection nodeB = connectToNode(2, 5772)) {
            Channel channelA = nodeA.createChannel();
            String queue = Support.declareQueue(channelA, "it-mn-get");
            channelA.basicPublish("", queue, null, "from-node-a".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(nodeB.createChannel(), queue, true);
            assertNotNull(resp);
            assertEquals("from-node-a", new String(resp.getBody(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void publishOnOneNodeConsumeOnAnotherNodeReceivesTheMessage() throws Exception {
        try (Connection nodeA = connectToNode(1, 5672);
                Connection nodeB = connectToNode(3, 5872)) {
            Channel channelA = nodeA.createChannel();
            String queue = Support.declareQueue(channelA, "it-mn-consume");

            CountDownLatch received = new CountDownLatch(1);
            AtomicReference<String> body = new AtomicReference<>();
            nodeB.createChannel().basicConsume(queue, true, (tag, delivery) -> {
                body.set(new String(delivery.getBody(), StandardCharsets.UTF_8));
                received.countDown();
            }, tag -> {});

            channelA.basicPublish("", queue, null, "cross-node-deliver".getBytes(StandardCharsets.UTF_8));

            assertTrue(received.await(15, TimeUnit.SECONDS));
            assertEquals("cross-node-deliver", body.get());
        }
    }

    @Test
    void consumersOnAllThreeNodesSplitOneQueueWithNoDuplicates() throws Exception {
        try (Connection nodeA = connectToNode(1, 5672);
                Connection nodeB = connectToNode(2, 5772);
                Connection nodeC = connectToNode(3, 5872)) {
            Channel setupChannel = nodeA.createChannel();
            String queue = Support.declareQueue(setupChannel, "it-mn-fanout");

            int total = 30;
            CountDownLatch allReceived = new CountDownLatch(total);
            AtomicInteger countA = new AtomicInteger();
            AtomicInteger countB = new AtomicInteger();
            AtomicInteger countC = new AtomicInteger();
            nodeA.createChannel().basicConsume(queue, true, (tag, d) -> {
                countA.incrementAndGet();
                allReceived.countDown();
            }, tag -> {});
            nodeB.createChannel().basicConsume(queue, true, (tag, d) -> {
                countB.incrementAndGet();
                allReceived.countDown();
            }, tag -> {});
            nodeC.createChannel().basicConsume(queue, true, (tag, d) -> {
                countC.incrementAndGet();
                allReceived.countDown();
            }, tag -> {});

            // let the consumer registrations propagate (meta-service raft write
            // + cross-node notify) before publishing.
            Thread.sleep(1000);

            Channel publishChannel = nodeB.createChannel();
            for (int i = 0; i < total; i++) {
                publishChannel.basicPublish("", queue, null, ("v" + i).getBytes(StandardCharsets.UTF_8));
            }

            assertTrue(allReceived.await(30, TimeUnit.SECONDS));
            assertEquals(total, countA.get() + countB.get() + countC.get());
            assertTrue(countA.get() > 0 && countB.get() > 0 && countC.get() > 0,
                    "A=" + countA.get() + " B=" + countB.get() + " C=" + countC.get());
        }
    }

    @Test
    void queueDeclaredOnOneNodeIsVisibleFromAnotherNode() throws Exception {
        try (Connection nodeA = connectToNode(2, 5772);
                Connection nodeB = connectToNode(3, 5872)) {
            String queue = Support.declareQueue(nodeA.createChannel(), "it-mn-declare");
            // throws if the queue isn't visible cluster-wide
            nodeB.createChannel().queueDeclarePassive(queue);
        }
    }
}
