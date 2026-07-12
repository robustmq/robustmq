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
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.function.BooleanSupplier;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.ConsumerConfig;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.consumer.ConsumerRecords;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.clients.consumer.OffsetAndMetadata;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
import org.junit.jupiter.api.Test;

/**
 * Part 1 of consumer verification: reading through a consumer group
 * {@code subscribe()} — exercises the group coordinator (join/sync/heartbeat),
 * offset commit/fetch and rebalancing on top of Fetch.
 */
class ConsumeGroupTest {

    private static String topicName() {
        return "it-group-" + UUID.randomUUID();
    }

    private static String groupId() {
        return "grp-" + UUID.randomUUID();
    }

    private static void createTopic(String topic, int partitions) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
        }
    }

    /** Produce {@code count} records round-robin across {@code partitions}. */
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

    /** Poll every consumer in turn until {@code predicate} holds or time runs out. */
    private static void pump(List<KafkaConsumer<byte[], byte[]>> consumers, BooleanSupplier done) {
        long deadline = System.nanoTime() + Duration.ofSeconds(30).toNanos();
        while (!done.getAsBoolean() && System.nanoTime() < deadline) {
            for (KafkaConsumer<byte[], byte[]> c : consumers) {
                c.poll(Duration.ofMillis(300));
            }
        }
    }

    // ---- single-consumer ---------------------------------------------------

    @Test
    void subscribeReadsAllProducedRecords() throws Exception {
        String topic = topicName();
        createTopic(topic, 1);
        produce(topic, 1, 5);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId())) {
            consumer.subscribe(List.of(topic));
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 5);
            assertEquals(5, records.size(), "group consumer should read every produced record");
            for (int i = 0; i < 5; i++) {
                assertEquals("v" + i, new String(records.get(i).value()));
            }
        }
    }

    @Test
    void multiPartitionSingleConsumerReadsAll() throws Exception {
        String topic = topicName();
        createTopic(topic, 3);
        produce(topic, 3, 9);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId())) {
            consumer.subscribe(List.of(topic));
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 9);
            assertEquals(9, records.size(), "one consumer should own all 3 partitions");
            Set<Integer> partitions = new HashSet<>();
            Set<String> values = new HashSet<>();
            for (ConsumerRecord<byte[], byte[]> r : records) {
                partitions.add(r.partition());
                values.add(new String(r.value()));
            }
            assertEquals(Set.of(0, 1, 2), partitions, "records span every partition");
            assertEquals(9, values.size(), "no record is dropped or duplicated");
        }
    }

    // ---- offsets -----------------------------------------------------------

    @Test
    void committedOffsetResumesForNewConsumerInSameGroup() throws Exception {
        String topic = topicName();
        TopicPartition tp = new TopicPartition(topic, 0);
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 1, 5);

        try (KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group)) {
            c1.subscribe(List.of(topic));
            assertEquals(5, drain(c1, 5).size());
            c1.commitSync(Map.of(tp, new OffsetAndMetadata(5L)));
            assertEquals(5L, c1.committed(Set.of(tp)).get(tp).offset(), "commit should persist");
        }

        produce(topic, 1, 5);
        try (KafkaConsumer<byte[], byte[]> c2 = Support.newConsumer(group)) {
            c2.subscribe(List.of(topic));
            List<ConsumerRecord<byte[], byte[]>> records = drain(c2, 5);
            assertEquals(5, records.size(), "should resume from committed offset, not re-read");
            assertTrue(records.get(0).offset() >= 5, "first resumed record is at/after commit");
        }
    }

    @Test
    void autoCommitPersistsOffsets() throws Exception {
        String topic = topicName();
        TopicPartition tp = new TopicPartition(topic, 0);
        String group = groupId();
        createTopic(topic, 1);
        produce(topic, 1, 5);

        Map<String, Object> autoCommit = Map.of(
                ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, "true",
                ConsumerConfig.AUTO_COMMIT_INTERVAL_MS_CONFIG, "200");
        try (KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group, autoCommit)) {
            c1.subscribe(List.of(topic));
            assertEquals(5, drain(c1, 5).size());
            // Auto-commit flushes on the interval and on close; give it a beat.
            Thread.sleep(500);
            c1.commitSync();
            assertEquals(5L, c1.committed(Set.of(tp)).get(tp).offset(),
                    "auto-commit should have advanced the committed offset");
        }
    }

    @Test
    void committedIsEmptyForNeverCommittedGroup() throws Exception {
        String topic = topicName();
        TopicPartition tp = new TopicPartition(topic, 0);
        createTopic(topic, 1);
        produce(topic, 1, 3);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId())) {
            // A group that has never committed has no stored offset for the partition.
            Map<TopicPartition, OffsetAndMetadata> committed = consumer.committed(Set.of(tp));
            assertTrue(committed.get(tp) == null, "uncommitted group must report no offset");
        }
    }

    // ---- reset & empty -----------------------------------------------------

    @Test
    void latestResetSkipsPreExistingRecords() throws Exception {
        String topic = topicName();
        createTopic(topic, 1);
        produce(topic, 1, 5); // exist before the consumer joins

        Map<String, Object> latest = Map.of(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "latest");
        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId(), latest)) {
            consumer.subscribe(List.of(topic));
            // Drive the initial assignment/position without expecting old data.
            pump(List.of(consumer), () -> !consumer.assignment().isEmpty());
            consumer.poll(Duration.ofMillis(500));

            produce(topic, 1, 3); // only these 3 (offsets 5..7) should be seen
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 3);
            assertEquals(3, records.size(), "latest reset must skip the 5 pre-existing records");
            assertTrue(records.get(0).offset() >= 5, "first record is past the pre-existing data");
        }
    }

    @Test
    void pollOnEmptyTopicReturnsNoRecords() throws Exception {
        String topic = topicName();
        createTopic(topic, 1);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(groupId())) {
            consumer.subscribe(List.of(topic));
            ConsumerRecords<byte[], byte[]> polled = ConsumerRecords.empty();
            long deadline = System.nanoTime() + Duration.ofSeconds(10).toNanos();
            int seen = 0;
            while (System.nanoTime() < deadline) {
                polled = consumer.poll(Duration.ofMillis(500));
                seen += polled.count();
            }
            assertEquals(0, seen, "an empty topic yields no records and does not hang");
        }
    }

    // ---- multi-consumer rebalancing ---------------------------------------

    @Test
    void twoConsumersInGroupSplitPartitions() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 2);
        produce(topic, 2, 6);

        try (KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group);
                KafkaConsumer<byte[], byte[]> c2 = Support.newConsumer(group)) {
            c1.subscribe(List.of(topic));
            c2.subscribe(List.of(topic));

            List<KafkaConsumer<byte[], byte[]>> both = List.of(c1, c2);
            List<ConsumerRecord<byte[], byte[]>> a = new ArrayList<>();
            List<ConsumerRecord<byte[], byte[]>> b = new ArrayList<>();
            long deadline = System.nanoTime() + Duration.ofSeconds(30).toNanos();
            while (a.size() + b.size() < 6 && System.nanoTime() < deadline) {
                for (ConsumerRecord<byte[], byte[]> r : c1.poll(Duration.ofMillis(300))) {
                    a.add(r);
                }
                for (ConsumerRecord<byte[], byte[]> r : c2.poll(Duration.ofMillis(300))) {
                    b.add(r);
                }
            }

            assertEquals(6, a.size() + b.size(), "together the group reads every record once");
            Set<Integer> pa = c1.assignment().stream().map(TopicPartition::partition)
                    .collect(java.util.stream.Collectors.toSet());
            Set<Integer> pb = c2.assignment().stream().map(TopicPartition::partition)
                    .collect(java.util.stream.Collectors.toSet());
            assertFalse(pa.isEmpty(), "consumer 1 should own a partition");
            assertFalse(pb.isEmpty(), "consumer 2 should own a partition");
            assertTrue(java.util.Collections.disjoint(pa, pb),
                    "the two consumers must not share a partition");
        }
    }

    @Test
    void memberLeaveReassignsPartitionsToSurvivor() throws Exception {
        String topic = topicName();
        String group = groupId();
        createTopic(topic, 2);

        KafkaConsumer<byte[], byte[]> c1 = Support.newConsumer(group);
        try {
            KafkaConsumer<byte[], byte[]> c2 = Support.newConsumer(group);
            c1.subscribe(List.of(topic));
            c2.subscribe(List.of(topic));
            // Let both settle onto one partition each.
            pump(List.of(c1, c2),
                    () -> c1.assignment().size() == 1 && c2.assignment().size() == 1);
            assertEquals(1, c1.assignment().size(), "precondition: c1 owns one partition");
            assertEquals(1, c2.assignment().size(), "precondition: c2 owns one partition");

            // c2 leaves the group; c1 must take over both partitions.
            c2.close();
            pump(List.of(c1), () -> c1.assignment().size() == 2);
            assertEquals(2, c1.assignment().size(),
                    "survivor should inherit both partitions after a member leaves");
        } finally {
            c1.close();
        }
    }

    // ---- fan-out across groups --------------------------------------------

    @Test
    void twoGroupsEachReadEveryRecord() throws Exception {
        String topic = topicName();
        createTopic(topic, 1);
        produce(topic, 1, 5);

        try (KafkaConsumer<byte[], byte[]> g1 = Support.newConsumer(groupId());
                KafkaConsumer<byte[], byte[]> g2 = Support.newConsumer(groupId())) {
            g1.subscribe(List.of(topic));
            g2.subscribe(List.of(topic));
            // Independent groups keep independent offsets, so each sees all five.
            assertEquals(5, drain(g1, 5).size(), "group 1 reads all records");
            assertEquals(5, drain(g2, 5).size(), "group 2 reads all records independently");
        }
    }
}
