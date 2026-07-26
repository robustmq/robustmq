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

import java.io.IOException;
import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicInteger;

import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.ConnectionFactory;
import com.rabbitmq.client.GetResponse;

/** Shared helpers for the RabbitMQ (AMQP 0-9-1) integration tests. */
final class Support {
    private Support() {}

    static String host() {
        String v = System.getProperty("amqp.host", System.getenv("AMQP_BROKER_HOST"));
        return v != null ? v : "127.0.0.1";
    }

    static int port() {
        String v = System.getProperty("amqp.port", System.getenv("AMQP_BROKER_PORT"));
        return v != null ? Integer.parseInt(v) : 5672; // matches scripts/cluster.sh's node 1
    }

    private static String user() {
        String v = System.getProperty("amqp.user", System.getenv("AMQP_BROKER_USER"));
        return v != null ? v : "admin";
    }

    private static String password() {
        String v = System.getProperty("amqp.password", System.getenv("AMQP_BROKER_PASSWORD"));
        return v != null ? v : "robustmq";
    }

    /** Vhost "" (not "/") maps to the broker's default tenant; see amqp-broker's Connection.Open. */
    static ConnectionFactory newFactory() {
        ConnectionFactory factory = new ConnectionFactory();
        factory.setHost(host());
        factory.setPort(port());
        factory.setUsername(user());
        factory.setPassword(password());
        factory.setVirtualHost("");
        return factory;
    }

    static Connection newConnection() throws Exception {
        return newFactory().newConnection();
    }

    static String uniqueName(String prefix) {
        return prefix + "-" + UUID.randomUUID();
    }

    static String declareQueue(Channel channel, String prefix) throws Exception {
        String name = uniqueName(prefix);
        channel.queueDeclare(name, true, false, false, null);
        return name;
    }

    static String declareDirectExchange(Channel channel, String prefix) throws Exception {
        String name = uniqueName(prefix);
        channel.exchangeDeclare(name, "direct", true);
        return name;
    }

    /** Polls basicGet (a pull API with no delivery callback) until a message shows up or times out. */
    static GetResponse pollGet(Channel channel, String queue, boolean autoAck) throws Exception {
        long deadline = System.nanoTime() + Duration.ofSeconds(10).toNanos();
        GetResponse resp;
        do {
            resp = channel.basicGet(queue, autoAck);
            if (resp != null) {
                return resp;
            }
            Thread.sleep(50);
        } while (System.nanoTime() < deadline);
        return null;
    }

    static void assertEventuallyEmpty(Channel channel, String queue) throws Exception {
        long deadline = System.nanoTime() + Duration.ofSeconds(5).toNanos();
        while (System.nanoTime() < deadline) {
            if (channel.basicGet(queue, true) == null) {
                return;
            }
            Thread.sleep(50);
        }
        throw new AssertionError("queue " + queue + " never became empty");
    }

    static void awaitCount(AtomicInteger counter, int target, int timeoutSeconds) throws InterruptedException {
        long deadline = System.nanoTime() + Duration.ofSeconds(timeoutSeconds).toNanos();
        while (counter.get() < target && System.nanoTime() < deadline) {
            Thread.sleep(20);
        }
        if (counter.get() < target) {
            throw new AssertionError("counter never reached " + target + ", stuck at " + counter.get());
        }
    }

    static boolean isConnectionRefused(Throwable t) {
        while (t != null) {
            if (t instanceof TimeoutException || t instanceof IOException) {
                return true;
            }
            t = t.getCause();
        }
        return false;
    }
}
