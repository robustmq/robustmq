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

import java.util.List;
import java.util.Set;
import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.junit.jupiter.api.Test;

// admin.listTopics() sends a Metadata request (API 3) for all topics, so this
// exercises process_metadata (as opposed to describeCluster/API 60 or
// describeTopics/API 75, which are covered by their own tests).
class MetadataTest {

    @Test
    void listTopicsReflectsCreatedTopic() throws Exception {
        String topic = "it-metadata-" + UUID.randomUUID();
        try (Admin admin = Support.newAdmin()) {
            Set<String> before = admin.listTopics().names().get();
            assertFalse(before.contains(topic), "topic unexpectedly present before creation");

            admin.createTopics(List.of(new NewTopic(topic, 2, (short) 1))).all().get();

            Set<String> after = admin.listTopics().names().get();
            assertFalse(after.isEmpty(), "metadata returned an empty topic list");
            assertTrue(after.contains(topic), "created topic missing from the metadata topic list");
        }
    }
}
