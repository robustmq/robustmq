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

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

import com.rabbitmq.client.AMQP;
import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.GetResponse;
import org.junit.jupiter.api.Test;

/** Basic.Publish routing: default exchange, direct exchange, mandatory returns, property round-trip. */
class PublishTest {

    @Test
    void defaultExchangeRoutesByQueueName() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-pub");

            channel.basicPublish("", queue, null, "hello".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, true);
            assertNotNull(resp);
            assertEquals("hello", new String(resp.getBody(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void directExchangeRoutesByBindingKey() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch");
            String queue = Support.declareQueue(channel, "it-queue");
            String routingKey = "orders.created";
            channel.queueBind(queue, exchange, routingKey);

            channel.basicPublish(exchange, routingKey, null, "order-1".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, true);
            assertNotNull(resp);
            assertEquals("order-1", new String(resp.getBody(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void directExchangeIgnoresNonMatchingRoutingKey() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch");
            String queue = Support.declareQueue(channel, "it-queue");
            channel.queueBind(queue, exchange, "orders.created");

            channel.basicPublish(exchange, "orders.cancelled", null, "x".getBytes(StandardCharsets.UTF_8));

            Support.assertEventuallyEmpty(channel, queue);
        }
    }

    @Test
    void mandatoryUnroutablePublishIsReturned() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch");

            CountDownLatch returned = new CountDownLatch(1);
            AtomicReference<String> returnedBody = new AtomicReference<>();
            AtomicReference<Integer> replyCode = new AtomicReference<>();
            channel.addReturnListener(returnMessage -> {
                returnedBody.set(new String(returnMessage.getBody(), StandardCharsets.UTF_8));
                replyCode.set(returnMessage.getReplyCode());
                returned.countDown();
            });

            channel.basicPublish(exchange, "no-such-binding", true, null,
                    "unroutable".getBytes(StandardCharsets.UTF_8));

            assertTrue(returned.await(10, TimeUnit.SECONDS));
            assertEquals("unroutable", returnedBody.get());
            assertEquals(312, replyCode.get()); // NO_ROUTE
        }
    }

    @Test
    void nonMandatoryUnroutablePublishIsSilentlyDropped() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch");

            CountDownLatch returned = new CountDownLatch(1);
            channel.addReturnListener(returnMessage -> returned.countDown());

            channel.basicPublish(exchange, "no-such-binding", false, null,
                    "dropped".getBytes(StandardCharsets.UTF_8));

            assertTrue(!returned.await(1, TimeUnit.SECONDS));
        }
    }

    @Test
    void fanoutExchangeDeliversToEveryBoundQueue() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.uniqueName("it-fanout");
            channel.exchangeDeclare(exchange, "fanout", true);
            String queueA = Support.declareQueue(channel, "it-fanout-a");
            String queueB = Support.declareQueue(channel, "it-fanout-b");
            channel.queueBind(queueA, exchange, "");
            channel.queueBind(queueB, exchange, "");

            channel.basicPublish(exchange, "ignored-routing-key", null, "broadcast".getBytes(StandardCharsets.UTF_8));

            GetResponse a = Support.pollGet(channel, queueA, true);
            GetResponse b = Support.pollGet(channel, queueB, true);
            assertNotNull(a);
            assertNotNull(b);
            assertEquals("broadcast", new String(a.getBody(), StandardCharsets.UTF_8));
            assertEquals("broadcast", new String(b.getBody(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void topicExchangeMatchesWildcardPattern() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.uniqueName("it-topic");
            channel.exchangeDeclare(exchange, "topic", true);
            String queue = Support.declareQueue(channel, "it-topic-queue");
            channel.queueBind(queue, exchange, "orders.*.created");

            channel.basicPublish(exchange, "orders.us.created", null, "match".getBytes(StandardCharsets.UTF_8));
            channel.basicPublish(exchange, "orders.us.cancelled", null, "no-match".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, true);
            assertNotNull(resp);
            assertEquals("match", new String(resp.getBody(), StandardCharsets.UTF_8));
            Support.assertEventuallyEmpty(channel, queue);
        }
    }

    @Test
    void headersExchangeMatchesOnDeclaredHeaders() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.uniqueName("it-headers");
            channel.exchangeDeclare(exchange, "headers", true);
            String queue = Support.declareQueue(channel, "it-headers-queue");

            Map<String, Object> bindArgs = new HashMap<>();
            bindArgs.put("x-match", "all");
            bindArgs.put("format", "pdf");
            channel.queueBind(queue, exchange, "", bindArgs);

            Map<String, Object> matching = new HashMap<>();
            matching.put("format", "pdf");
            channel.basicPublish(exchange, "", new AMQP.BasicProperties.Builder().headers(matching).build(),
                    "match".getBytes(StandardCharsets.UTF_8));

            Map<String, Object> nonMatching = new HashMap<>();
            nonMatching.put("format", "csv");
            channel.basicPublish(exchange, "", new AMQP.BasicProperties.Builder().headers(nonMatching).build(),
                    "no-match".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, true);
            assertNotNull(resp);
            assertEquals("match", new String(resp.getBody(), StandardCharsets.UTF_8));
            Support.assertEventuallyEmpty(channel, queue);
        }
    }

    @Test
    void contentTypeAndHeadersRoundTrip() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-props");

            Map<String, Object> headers = new HashMap<>();
            headers.put("trace-id", "abc-123");
            AMQP.BasicProperties props = new AMQP.BasicProperties.Builder()
                    .contentType("application/json")
                    .headers(headers)
                    .build();
            channel.basicPublish("", queue, props, "{}".getBytes(StandardCharsets.UTF_8));

            GetResponse resp = Support.pollGet(channel, queue, true);
            assertNotNull(resp);
            assertEquals("application/json", resp.getProps().getContentType());
            assertEquals("abc-123", String.valueOf(resp.getProps().getHeaders().get("trace-id")));
            assertArrayEquals("{}".getBytes(StandardCharsets.UTF_8), resp.getBody());
        }
    }
}
