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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.TopicDescription;
import org.apache.kafka.common.errors.UnknownTopicOrPartitionException;
import org.junit.jupiter.api.Test;

// admin.describeTopics() uses the DescribeTopicPartitions API (key 75) once the
// broker advertises it, so these exercise process_describe_topic_partitions.
class DescribeTopicPartitionsTest {

    @Test
    void describeMultiPartitionTopic() throws Exception {
        String topic = "it-dtp-" + UUID.randomUUID();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 4, (short) 1))).all().get();

            TopicDescription desc =
                    admin.describeTopics(List.of(topic)).allTopicNames().get().get(topic);

            assertEquals(4, desc.partitions().size(), "unexpected partition count");
            // Partitions must be indexed 0..n-1 and each must have a leader.
            for (int i = 0; i < 4; i++) {
                assertEquals(i, desc.partitions().get(i).partition(), "partitions out of order");
                assertTrue(desc.partitions().get(i).leader().id() >= 0, "partition has no leader");
            }
        }
    }

    @Test
    void describeUnknownTopicFails() throws Exception {
        String missing = "it-dtp-missing-" + UUID.randomUUID();
        try (Admin admin = Support.newAdmin()) {
            ExecutionException ex = assertThrows(
                    ExecutionException.class,
                    () -> admin.describeTopics(List.of(missing)).allTopicNames().get());
            assertTrue(
                    ex.getCause() instanceof UnknownTopicOrPartitionException,
                    "expected UnknownTopicOrPartition, got " + ex.getCause());
        }
    }

    @Test
    void describeMultipleTopicsAtOnce() throws Exception {
        String a = "it-dtp-a-" + UUID.randomUUID();
        String b = "it-dtp-b-" + UUID.randomUUID();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(
                    new NewTopic(a, 1, (short) 1),
                    new NewTopic(b, 3, (short) 1))).all().get();

            Map<String, TopicDescription> descs =
                    admin.describeTopics(List.of(a, b)).allTopicNames().get();

            assertEquals(1, descs.get(a).partitions().size(), "topic a partition count");
            assertEquals(3, descs.get(b).partitions().size(), "topic b partition count");
        }
    }
}
