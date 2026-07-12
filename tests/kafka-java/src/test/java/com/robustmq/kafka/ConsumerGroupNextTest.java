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

import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.function.BooleanSupplier;
import java.util.stream.Collectors;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.ConsumerGroupDescription;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
import org.junit.jupiter.api.Test;

/**
 * Phase 6 (KIP-848): the next-generation consumer group protocol —
 * ConsumerGroupHeartbeat(68) with server-side assignment, and
 * ConsumerGroupDescribe(69). Consumers opt in with {@code group.protocol=consumer}.
 */
class ConsumerGroupNextTest {

    private static final Map<String, Object> NEXT_GEN = Map.of("group.protocol", "consumer");

    private static String topicName() {
        return "it-next-" + UUID.randomUUID();
    }

    private static String groupId() {
        return "next-" + UUID.randomUUID();
    }

    private static void createTopic(String topic, int partitions) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
        }
    }

    private static void produce(String topic, int partitions, int count) throws Exception {
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            for (int i = 0; i < count; i++) {
                producer.send(new ProducerRecord<>(topic, i % partitions, null, ("v" + i).getBytes()))
                        .get();
            }
            producer.flush();
        }
    }

    private static List<ConsumerRecord<byte[], byte[]>> drain(
            KafkaConsumer<byte[], byte[]> consumer, int expected) {
        List<ConsumerRecord<byte[], byte[]>> out = new ArrayList<>();
        long deadline = System.nanoTime() + Duration.ofSeconds(30).toNanos();
        while (out.size() < expected && System.nanoTime() < deadline) {
            for (ConsumerRecord<byte[], byte[]> r : consumer.poll(Duration.ofMillis(500))) {
                out.add(r);
            }
        }
        return out;
    }

    @Test
    void nextGenConsumerReadsAllRecords() throws Exception {
        String topic = topicName();
        createTopic(topic, 1);
        produce(topic, 1, 5);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId(), NEXT_GEN)) {
            consumer.subscribe(List.of(topic));
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 5);
            assertEquals(5, records.size(), "the KIP-848 consumer should read every record");
            for (int i = 0; i < 5; i++) {
                assertEquals("v" + i, new String(records.get(i).value()));
            }
        }
    }

    @Test
    void nextGenSingleConsumerOwnsAllPartitions() throws Exception {
        String topic = topicName();
        createTopic(topic, 3);
        produce(topic, 3, 9);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId(), NEXT_GEN)) {
            consumer.subscribe(List.of(topic));
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 9);
            assertEquals(9, records.size(), "server-side assignment should give one consumer all 3 partitions");
            Set<Integer> partitions = new HashSet<>();
            for (ConsumerRecord<byte[], byte[]> r : records) {
                partitions.add(r.partition());
            }
            assertEquals(Set.of(0, 1, 2), partitions, "records span every partition");
        }
    }

    @Test
    void describeNextGenGroupShowsMember() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 1, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(group, NEXT_GEN)) {
            consumer.subscribe(List.of(topic));
            assertEquals(3, drain(consumer, 3).size());

            try (Admin admin = Support.newAdmin()) {
                ConsumerGroupDescription desc =
                        admin.describeConsumerGroups(List.of(group)).all().get().get(group);
                assertEquals(group, desc.groupId());
                assertEquals(1, desc.members().size(),
                        "ConsumerGroupDescribe should report the live member");
                assertTrue(desc.members().iterator().next().assignment().topicPartitions().size() >= 1,
                        "the member should have a server-assigned partition");
            }
        }
    }

    // ---- multi-consumer server-side reconciliation ------------------------

    private static void pump(List<KafkaConsumer<byte[], byte[]>> consumers, BooleanSupplier done) {
        long deadline = System.nanoTime() + Duration.ofSeconds(30).toNanos();
        while (!done.getAsBoolean() && System.nanoTime() < deadline) {
            for (KafkaConsumer<byte[], byte[]> c : consumers) {
                c.poll(Duration.ofMillis(300));
            }
        }
    }

    private static Set<Integer> owned(KafkaConsumer<byte[], byte[]> c) {
        return c.assignment().stream().map(TopicPartition::partition).collect(Collectors.toSet());
    }

    @Test
    void twoNextGenConsumersSplitPartitions() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 2);
        produce(topic, 2, 6);

        try (KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group, NEXT_GEN);
                KafkaConsumer<byte[], byte[]> c2 = Support.newConsumer(group, NEXT_GEN)) {
            c1.subscribe(List.of(topic));
            c2.subscribe(List.of(topic));

            // KIP-848 reconciles incrementally: c1 may briefly own both partitions
            // before the coordinator moves one to c2, so wait for both members to
            // hold an assignment rather than stopping at a record count.
            pump(List.of(c1, c2), () -> !owned(c1).isEmpty() && !owned(c2).isEmpty());

            Set<Integer> pa = owned(c1);
            Set<Integer> pb = owned(c2);
            assertFalse(pa.isEmpty(), "consumer 1 should be assigned a partition");
            assertFalse(pb.isEmpty(), "consumer 2 should be assigned a partition");
            assertTrue(Collections.disjoint(pa, pb),
                    "server-side assignment must not overlap partitions across members");
            Set<Integer> union = new HashSet<>(pa);
            union.addAll(pb);
            assertEquals(Set.of(0, 1), union, "both partitions must be assigned across the group");
        }
    }

    @Test
    void nextGenMemberLeaveReassignsToSurvivor() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 2);

        KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group, NEXT_GEN);
        try {
            KafkaConsumer<byte[], byte[]> c2 = Support.newConsumer(group, NEXT_GEN);
            c1.subscribe(List.of(topic));
            c2.subscribe(List.of(topic));
            pump(List.of(c1, c2), () -> owned(c1).size() == 1 && owned(c2).size() == 1);
            assertEquals(1, owned(c1).size(), "precondition: c1 owns one partition");
            assertEquals(1, owned(c2).size(), "precondition: c2 owns one partition");

            // One member leaves; the coordinator must reassign both partitions to
            // the survivor on the next heartbeats.
            c2.close();
            pump(List.of(c1), () -> owned(c1).size() == 2);
            assertEquals(2, owned(c1).size(),
                    "survivor should inherit both partitions after a KIP-848 member leaves");
        } finally {
            c1.close();
        }
    }
}
