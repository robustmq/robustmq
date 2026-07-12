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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.ConsumerGroupDescription;
import org.apache.kafka.clients.admin.ConsumerGroupListing;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.clients.consumer.OffsetAndMetadata;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
import org.junit.jupiter.api.Test;

/**
 * Phase 1: consumer-group management/introspection — ListGroups(16),
 * DescribeGroups(15), DeleteGroups(42) and OffsetDelete(47) via the AdminClient.
 */
class ConsumeGroupAdminTest {

    private static String topicName() {
        return "it-grpadm-" + UUID.randomUUID();
    }

    private static String groupId() {
        return "grpadm-" + UUID.randomUUID();
    }

    private static void createTopic(String topic, int partitions) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
        }
    }

    private static void produce(String topic, int count) throws Exception {
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            for (int i = 0; i < count; i++) {
                producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes())).get();
            }
            producer.flush();
        }
    }

    /** Subscribe and poll until the consumer owns a partition (i.e. has joined). */
    private static void joinAndDrain(KafkaConsumer<byte[], byte[]> consumer, String topic, int expected) {
        consumer.subscribe(List.of(topic));
        int seen = 0;
        long deadline = System.nanoTime() + Duration.ofSeconds(30).toNanos();
        while ((consumer.assignment().isEmpty() || seen < expected) && System.nanoTime() < deadline) {
            seen += consumer.poll(Duration.ofMillis(300)).count();
        }
    }

    @Test
    void listAndDescribeActiveGroup() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(group)) {
            joinAndDrain(consumer, topic, 3);

            try (Admin admin = Support.newAdmin()) {
                Set<String> listed = admin.listConsumerGroups().all().get().stream()
                        .map(ConsumerGroupListing::groupId).collect(Collectors.toSet());
                assertTrue(listed.contains(group), "an active group must be listed: " + listed);

                ConsumerGroupDescription desc =
                        admin.describeConsumerGroups(List.of(group)).all().get().get(group);
                assertEquals(group, desc.groupId());
                assertEquals(1, desc.members().size(), "the one live member should be described");
                assertFalse(desc.state().toString().equalsIgnoreCase("DEAD"),
                        "an active group is not Dead, was " + desc.state());
            }
        }
    }

    @Test
    void describeUnknownGroupIsDeadNotAnError() throws Exception {
        String group = groupId(); // never used
        try (Admin admin = Support.newAdmin()) {
            ConsumerGroupDescription desc =
                    admin.describeConsumerGroups(List.of(group)).all().get().get(group);
            assertTrue(desc.members().isEmpty(), "unknown group has no members");
            assertEquals("DEAD", desc.state().toString().toUpperCase(),
                    "describing an unknown group reports Dead, not an error");
        }
    }

    @Test
    void deleteActiveGroupIsRejected() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(group)) {
            joinAndDrain(consumer, topic, 3);
            try (Admin admin = Support.newAdmin()) {
                ExecutionException ex = assertThrows(ExecutionException.class,
                        () -> admin.deleteConsumerGroups(List.of(group)).all().get());
                assertTrue(ex.getCause() instanceof org.apache.kafka.common.errors.GroupNotEmptyException,
                        "deleting a group with live members must fail, got " + ex.getCause());
            }
        }
    }

    @Test
    void deleteEmptyGroupRemovesIt() throws Exception {
        String topic = topicName();
        TopicPartition tp = new TopicPartition(topic, 0);
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(group)) {
            joinAndDrain(consumer, topic, 3);
            consumer.commitSync(Map.of(tp, new OffsetAndMetadata(3L)));
        } // consumer closed -> group becomes Empty

        try (Admin admin = Support.newAdmin()) {
            admin.deleteConsumerGroups(List.of(group)).all().get();
            Set<String> listed = admin.listConsumerGroups().all().get().stream()
                    .map(ConsumerGroupListing::groupId).collect(Collectors.toSet());
            assertFalse(listed.contains(group), "a deleted group must no longer be listed");
        }
    }

    @Test
    void deleteConsumerGroupOffsetsClearsCommit() throws Exception {
        String topic = topicName();
        TopicPartition tp = new TopicPartition(topic, 0);
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(group)) {
            joinAndDrain(consumer, topic, 3);
            consumer.commitSync(Map.of(tp, new OffsetAndMetadata(3L)));
            assertEquals(3L, consumer.committed(Set.of(tp)).get(tp).offset(), "precondition: committed");

            try (Admin admin = Support.newAdmin()) {
                admin.deleteConsumerGroupOffsets(group, Set.of(tp)).all().get();
            }
            assertNull(consumer.committed(Set.of(tp)).get(tp),
                    "OffsetDelete should remove the committed offset");
        }
    }
}
