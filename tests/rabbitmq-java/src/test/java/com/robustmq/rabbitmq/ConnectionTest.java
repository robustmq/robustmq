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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;

import com.rabbitmq.client.AlreadyClosedException;
import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.ConnectionFactory;
import org.junit.jupiter.api.Test;

/** Connection/Channel handshake and lifecycle. */
class ConnectionTest {

    @Test
    void validCredentialsOpenAConnection() throws Exception {
        try (Connection connection = Support.newConnection()) {
            assertTrue(connection.isOpen());
        }
    }

    @Test
    void wrongPasswordIsRejected() {
        ConnectionFactory factory = Support.newFactory();
        factory.setPassword(factory.getPassword() + "-wrong");
        Exception ex = assertThrows(IOException.class, factory::newConnection);
        assertTrue(Support.isConnectionRefused(ex), "expected an auth/IO failure, got " + ex);
    }

    @Test
    void unknownVirtualHostIsRejected() {
        ConnectionFactory factory = Support.newFactory();
        factory.setVirtualHost("/does-not-exist");
        assertThrows(IOException.class, factory::newConnection);
    }

    @Test
    void connectionSurvivesMultipleChannels() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel a = connection.createChannel();
            Channel b = connection.createChannel();
            assertTrue(a.isOpen());
            assertTrue(b.isOpen());
            assertTrue(a.getChannelNumber() != b.getChannelNumber());
            a.close();
            assertFalse(a.isOpen());
            assertTrue(connection.isOpen(), "closing one channel must not close the connection");
            assertTrue(b.isOpen());
        }
    }

    @Test
    void closedChannelRejectsFurtherUse() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            channel.close();
            assertThrows(AlreadyClosedException.class,
                    () -> channel.queueDeclare(Support.uniqueName("it-queue"), true, false, false, null));
        }
    }

    @Test
    void queueAndExchangeDeclareSucceed() throws Exception {
        try (Connection connection = Support.newConnection()) {
            Channel channel = connection.createChannel();
            String queue = Support.uniqueName("it-queue");
            String exchange = Support.uniqueName("it-exchange");
            assertEquals(queue, channel.queueDeclare(queue, true, false, false, null).getQueue());
            channel.exchangeDeclare(exchange, "direct", true);
            channel.queueBind(queue, exchange, "rk");
        }
    }

    @Test
    void connectionCloseIsClean() throws Exception {
        Connection connection = Support.newConnection();
        Channel channel = connection.createChannel();
        assertTrue(channel.isOpen());
        connection.close();
        assertFalse(connection.isOpen());
    }
}
