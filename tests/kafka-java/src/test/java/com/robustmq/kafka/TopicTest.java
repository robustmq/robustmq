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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.TopicDescription;
import org.junit.jupiter.api.Test;

class TopicTest {

    @Test
    void createTopicThenAppearsInList() throws Exception {
        String topic = "it-topic-" + UUID.randomUUID();
        try (Admin admin = Support.newAdmin()) {
            assertFalse(admin.listTopics().names().get().contains(topic),
                    "topic unexpectedly present before creation");

            admin.createTopics(List.of(new NewTopic(topic, 3, (short) 1))).all().get();

            assertTrue(admin.listTopics().names().get().contains(topic),
                    "topic missing from list after creation");

            TopicDescription desc = admin.describeTopics(List.of(topic)).allTopicNames().get().get(topic);
            assertEquals(3, desc.partitions().size(), "unexpected partition count");
        }
    }
}
