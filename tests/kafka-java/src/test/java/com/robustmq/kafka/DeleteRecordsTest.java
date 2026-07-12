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
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.RecordsToDelete;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
import org.apache.kafka.common.errors.OffsetOutOfRangeException;
import org.apache.kafka.common.errors.UnknownTopicOrPartitionException;
import org.junit.jupiter.api.Test;

class DeleteRecordsTest {

    private static String name() {
        return "it-delrec-" + UUID.randomUUID();
    }

    private static void produce(String topic, int count) throws Exception {
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            for (int i = 0; i < count; i++) {
                producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes())).get();
            }
            producer.flush();
        }
    }

    @Test
    void deleteBeforeOffsetAdvancesLowWatermark() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5);

            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(3L)));
            long low = result.lowWatermarks().get(tp).get().lowWatermark();
            assertEquals(3L, low, "low watermark should advance to the delete offset");
        }
    }

    @Test
    void deleteAllWithHighWatermarkSentinel() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5);

            // beforeOffset(-1) is the "delete up to the high watermark" sentinel.
            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(-1L)));
            long low = result.lowWatermarks().get(tp).get().lowWatermark();
            assertEquals(5L, low, "low watermark should advance to the high watermark");
        }
    }

    @Test
    void deleteExactlyAtHighWatermarkAdvancesToHighWatermark() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5);

            // Explicit offset == HW is valid (delete everything), unlike offset > HW.
            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(5L)));
            assertEquals(5L, result.lowWatermarks().get(tp).get().lowWatermark());
        }
    }

    @Test
    void deleteAtZeroIsANoOp() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5);

            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(0L)));
            assertEquals(0L, result.lowWatermarks().get(tp).get().lowWatermark(),
                    "deleting before offset 0 removes nothing");
        }
    }

    @Test
    void deleteBeyondHighWatermarkReturnsOffsetOutOfRange() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5); // high watermark = 5

            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(10L)));
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> result.lowWatermarks().get(tp).get());
            assertInstanceOf(OffsetOutOfRangeException.class, ex.getCause(),
                    "deleting past the high watermark must be rejected");
        }
    }

    @Test
    void deleteOnUnknownTopicFails() throws Exception {
        String topic = name(); // never created
        TopicPartition tp = new TopicPartition(topic, 0);
        // The admin client resolves the partition leader via metadata before it
        // ever sends DeleteRecords; a missing topic yields a (retriable)
        // UNKNOWN_TOPIC_OR_PARTITION, so the call fails with either that error or
        // a metadata timeout — both mean "the topic isn't there". Short timeout
        // so the retget doesn't sit for the 60s default.
        Map<String, Object> fastTimeout = Map.of(
                org.apache.kafka.clients.admin.AdminClientConfig.REQUEST_TIMEOUT_MS_CONFIG, 4000,
                org.apache.kafka.clients.admin.AdminClientConfig.DEFAULT_API_TIMEOUT_MS_CONFIG, 8000);
        try (Admin admin = Support.newAdmin(fastTimeout)) {
            var result = admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(1L)));
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> result.lowWatermarks().get(tp).get());
            Throwable cause = ex.getCause();
            assertTrue(cause instanceof UnknownTopicOrPartitionException
                            || cause instanceof org.apache.kafka.common.errors.TimeoutException,
                    "expected unknown-topic or metadata timeout, got " + cause);
        }
    }

    @Test
    void consumeAfterDeleteStartsAtNewLowWatermark() throws Exception {
        String topic = name();
        TopicPartition tp = new TopicPartition(topic, 0);
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            produce(topic, 5);
            admin.deleteRecords(Map.of(tp, RecordsToDelete.beforeOffset(3L)))
                    .lowWatermarks().get(tp).get();
        }
        // After truncation the readable log begins at the new low watermark (3),
        // so seek-to-beginning must land on offset 3, not 0.
        List<ConsumerRecord<byte[], byte[]>> records = Support.consumeAllFromBeginning(topic, 2);
        assertTrue(records.size() >= 1, "records at/after the low watermark remain readable");
        assertEquals(3L, records.get(0).offset(), "first readable record is the low watermark");
        assertEquals("v3", new String(records.get(0).value()));
    }
}
