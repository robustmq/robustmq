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
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.time.Duration;

import com.rabbitmq.client.AMQP;
import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import org.junit.jupiter.api.Test;

/** Passive declare semantics and real (non-zero-stub) Declare/Delete accounting. */
class DeclareSemanticsTest {

    @Test
    void exchangePassiveDeclareOnMissingExchangeThrows404() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            assertThrows(IOException.class,
                    () -> channel.exchangeDeclarePassive(Support.uniqueName("it-missing-exchange")));
        }
    }

    @Test
    void exchangePassiveDeclareSucceedsForExistingExchange() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch-passive");
            channel.exchangeDeclarePassive(exchange); // must not throw
        }
    }

    @Test
    void queueDeclareReportsRealMessageCount() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-declare-count");
            channel.basicPublish("", queue, null, "a".getBytes(StandardCharsets.UTF_8));
            channel.basicPublish("", queue, null, "b".getBytes(StandardCharsets.UTF_8));

            AMQP.Queue.DeclareOk declareOk = awaitMessageCount(channel, queue, 2);
            assertEquals(2, declareOk.getMessageCount());
        }
    }

    @Test
    void queueDeclareReportsRealConsumerCount() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-declare-consumers");
            channel.basicConsume(queue, true, (tag, delivery) -> {}, tag -> {});

            AMQP.Queue.DeclareOk declareOk = awaitConsumerCount(channel, queue, 1);
            assertEquals(1, declareOk.getConsumerCount());
        }
    }

    @Test
    void queuePurgeRemovesEveryMessageIncludingTheNewest() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.declareQueue(channel, "it-purge");
            channel.basicPublish("", queue, null, "a".getBytes(StandardCharsets.UTF_8));
            channel.basicPublish("", queue, null, "b".getBytes(StandardCharsets.UTF_8));
            awaitMessageCount(channel, queue, 2);

            channel.queuePurge(queue);

            Support.assertEventuallyEmpty(channel, queue);
        }
    }

    @Test
    void exchangeDeleteIfUnusedFailsWhenStillBound() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String exchange = Support.declareDirectExchange(channel, "it-exch-unused");
            String queue = Support.declareQueue(channel, "it-queue-unused");
            channel.queueBind(queue, exchange, "rk");

            assertThrows(IOException.class, () -> channel.exchangeDelete(exchange, true));
        }
    }

    @Test
    void queueDeleteIfUnusedFailsWhenConsumerAttached() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel consumeChannel = connection.createChannel();
            String queue = Support.declareQueue(consumeChannel, "it-queue-consumer-attached");
            consumeChannel.basicConsume(queue, true, (tag, delivery) -> {}, tag -> {});
            awaitConsumerCount(consumeChannel, queue, 1);

            Channel deleteChannel = connection.createChannel();
            assertThrows(IOException.class, () -> deleteChannel.queueDelete(queue, true, false));
        }
    }

    /** consumer_count/message_count come from a replicated cache, so poll briefly instead of asserting immediately. */
    private static AMQP.Queue.DeclareOk awaitMessageCount(Channel channel, String queue, int target) throws Exception {
        long deadline = System.nanoTime() + Duration.ofSeconds(5).toNanos();
        AMQP.Queue.DeclareOk last;
        do {
            last = channel.queueDeclarePassive(queue);
            if (last.getMessageCount() >= target) {
                return last;
            }
            Thread.sleep(50);
        } while (System.nanoTime() < deadline);
        return last;
    }

    private static AMQP.Queue.DeclareOk awaitConsumerCount(Channel channel, String queue, int target) throws Exception {
        long deadline = System.nanoTime() + Duration.ofSeconds(5).toNanos();
        AMQP.Queue.DeclareOk last;
        do {
            last = channel.queueDeclarePassive(queue);
            if (last.getConsumerCount() >= target) {
                return last;
            }
            Thread.sleep(50);
        } while (System.nanoTime() < deadline);
        return last;
    }
}
