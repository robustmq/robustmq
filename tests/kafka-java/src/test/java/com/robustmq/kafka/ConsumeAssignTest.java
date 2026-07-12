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
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.consumer.ConsumerRecords;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
import org.apache.kafka.common.header.Header;
import org.apache.kafka.common.header.internals.RecordHeader;
import org.junit.jupiter.api.Test;

/**
 * Part 2 of consumer verification: reading with an explicit partition
 * {@code assign()} — no group coordinator involved, just Metadata + ListOffsets
 * + Fetch.
 */
class ConsumeAssignTest {

    private static String name() {
        return "it-assign-" + UUID.randomUUID();
    }

    /** Produce {@code count} records carrying a key, value and one header each. */
    private static void produce(String topic, int count) throws Exception {
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            for (int i = 0; i < count; i++) {
                ProducerRecord<byte[], byte[]> record =
                        new ProducerRecord<>(topic, 0, ("k" + i).getBytes(), ("v" + i).getBytes());
                record.headers().add(new RecordHeader("idx", Integer.toString(i).getBytes()));
                producer.send(record).get();
            }
            producer.flush();
        }
    }

    /** Poll until {@code expected} records are drained or the deadline passes. */
    private static List<ConsumerRecord<byte[], byte[]>> drain(
            KafkaConsumer<byte[], byte[]> consumer, int expected) {
        List<ConsumerRecord<byte[], byte[]>> out = new ArrayList<>();
        long deadline = System.nanoTime() + Duration.ofSeconds(15).toNanos();
        while (out.size() < expected && System.nanoTime() < deadline) {
            ConsumerRecords<byte[], byte[]> polled = consumer.poll(Duration.ofMillis(500));
            for (ConsumerRecord<byte[], byte[]> r : polled) {
                out.add(r);
            }
        }
        return out;
    }

    @Test
    void assignReadsAllProducedRecordsInOrder() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
        }
        produce(topic, 5);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(null)) {
            consumer.assign(List.of(tp));
            consumer.seekToBeginning(List.of(tp));
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 5);

            assertEquals(5, records.size(), "should read back every produced record");
            for (int i = 0; i < 5; i++) {
                ConsumerRecord<byte[], byte[]> r = records.get(i);
                assertEquals(i, r.offset(), "records must arrive in offset order");
                assertEquals("k" + i, new String(r.key()), "key must round-trip");
                assertEquals("v" + i, new String(r.value()), "value must round-trip");
                Header h = r.headers().lastHeader("idx");
                assertTrue(h != null && Integer.toString(i).equals(new String(h.value())),
                        "header must round-trip");
            }
        }
    }

    @Test
    void beginningAndEndOffsetsReflectLog() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
        }
        produce(topic, 5);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(null)) {
            consumer.assign(List.of(tp));
            // ListOffsets earliest/latest sentinels.
            Map<TopicPartition, Long> begin = consumer.beginningOffsets(List.of(tp));
            Map<TopicPartition, Long> end = consumer.endOffsets(List.of(tp));
            assertEquals(0L, begin.get(tp), "log starts at offset 0");
            assertEquals(5L, end.get(tp), "high watermark is one past the last record");
        }
    }

    @Test
    void seekReadsFromRequestedOffset() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
        }
        produce(topic, 5);

        try (KafkaConsumer<byte[], byte[]> consumer = Support.newConsumer(null)) {
            consumer.assign(List.of(tp));
            consumer.seek(tp, 2L);
            List<ConsumerRecord<byte[], byte[]>> records = drain(consumer, 3);

            assertEquals(3, records.size(), "seeking to offset 2 should skip the first two");
            assertEquals(2L, records.get(0).offset());
            assertEquals("v2", new String(records.get(0).value()));
            assertEquals("v4", new String(records.get(2).value()));
        }
    }
}
