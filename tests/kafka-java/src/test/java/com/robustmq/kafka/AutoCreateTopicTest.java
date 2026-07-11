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
package com.robustmq.kafka;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.junit.jupiter.api.Test;

class AutoCreateTopicTest {

    private boolean topicExists(String topic) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            return admin.listTopics().names().get().contains(topic);
        }
    }

    @Test
    void metadataAutoCreatesTopicWhenEnabled() throws Exception {
        String topic = "it-autocreate-" + UUID.randomUUID();

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newAutoCreateConsumer()) {
            // Switch off: a metadata lookup for the unknown topic must not create it.
            Support.setAutoCreateTopics(false);
            consumer.partitionsFor(topic);
            assertFalse(topicExists(topic), "topic was created while auto-create was disabled");

            // Switch on: the same lookup should now materialize the topic.
            Support.setAutoCreateTopics(true);
            boolean created = false;
            for (int i = 0; i < 10; i++) {
                consumer.partitionsFor(topic);
                if (topicExists(topic)) {
                    created = true;
                    break;
                }
                Thread.sleep(500);
            }

            Support.setAutoCreateTopics(false);
            assertTrue(created, "topic was not auto-created after enabling the switch");
        }
    }
}
