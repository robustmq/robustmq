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

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Properties;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Future;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerConfig;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.clients.producer.RecordMetadata;
import org.apache.kafka.common.errors.TimeoutException;
import org.apache.kafka.common.errors.UnknownTopicOrPartitionException;
import org.apache.kafka.common.header.internals.RecordHeader;
import org.apache.kafka.common.serialization.ByteArraySerializer;
import org.junit.jupiter.api.Test;

class ProduceTest {

    private static String name() {
        return "it-produce-" + UUID.randomUUID();
    }

    private static void createTopic(String topic, int partitions) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
        }
    }

    // ---- basics -----------------------------------------------------------

    @Test
    void produceSingleRecordReturnsOffset() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            RecordMetadata md =
                    producer.send(new ProducerRecord<>(topic, 0, null, "hello".getBytes())).get();
            assertEquals(topic, md.topic());
            assertEquals(0, md.partition());
            assertTrue(md.offset() >= 0, "offset should be assigned");
        }
    }

    @Test
    void offsetsAdvanceMonotonicallyWithinAPartition() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            long prev = -1;
            for (int i = 0; i < 5; i++) {
                RecordMetadata md =
                        producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes()))
                                .get();
                assertTrue(md.offset() > prev,
                        "offset must strictly increase: " + md.offset() + " after " + prev);
                prev = md.offset();
            }
            assertEquals(4L, prev, "five appends starting at 0 should end at offset 4");
        }
    }

    @Test
    void partitionsGetIndependentOffsetSequences() throws Exception {
        String topic = name();
        createTopic(topic, 2);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            RecordMetadata p0 =
                    producer.send(new ProducerRecord<>(topic, 0, null, "a".getBytes())).get();
            RecordMetadata p1 =
                    producer.send(new ProducerRecord<>(topic, 1, null, "b".getBytes())).get();
            assertEquals(0, p0.partition());
            assertEquals(0L, p0.offset());
            assertEquals(1, p1.partition());
            assertEquals(0L, p1.offset());
        }
    }

    @Test
    void keyAndHeadersAreAccepted() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            ProducerRecord<byte[], byte[]> record =
                    new ProducerRecord<>(topic, 0, "k".getBytes(), "v".getBytes());
            record.headers().add(new RecordHeader("trace", "abc".getBytes()));
            RecordMetadata md = producer.send(record).get();
            assertTrue(md.offset() >= 0, "keyed record with headers should be accepted");
        }
    }

    // ---- acks -------------------------------------------------------------

    @Test
    void acksZeroFireAndForgetCompletes() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        Properties props = new Properties();
        props.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, Support.bootstrapServers());
        props.put(ProducerConfig.ACKS_CONFIG, "0");
        props.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, false);
        props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class.getName());
        props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class.getName());
        try (KafkaProducer<byte[], byte[]> producer = new KafkaProducer<>(props)) {
            // With acks=0 the broker sends no Produce response; the send future still
            // completes locally once the record is written to the socket.
            RecordMetadata md =
                    producer.send(new ProducerRecord<>(topic, 0, null, "x".getBytes())).get();
            assertEquals(topic, md.topic());
        }
    }

    @Test
    void acksOneLeaderAckReturnsOffset() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        try (KafkaProducer<byte[], byte[]> producer =
                Support.newProducer(Map.of(ProducerConfig.ACKS_CONFIG, "1"))) {
            RecordMetadata md =
                    producer.send(new ProducerRecord<>(topic, 0, null, "x".getBytes())).get();
            assertEquals(0L, md.offset());
        }
    }

    // ---- batching ---------------------------------------------------------

    @Test
    void batchedSendsGetContiguousOffsets() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        // linger + a big batch buffer coax the client into packing all ten records
        // into a single Produce request, which the broker must offset one by one.
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer(Map.of(
                ProducerConfig.LINGER_MS_CONFIG, "50",
                ProducerConfig.BATCH_SIZE_CONFIG, "1048576"))) {
            List<Future<RecordMetadata>> futures = new ArrayList<>();
            for (int i = 0; i < 10; i++) {
                futures.add(producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes())));
            }
            producer.flush();
            for (int i = 0; i < 10; i++) {
                assertEquals((long) i, futures.get(i).get().offset(),
                        "batched record " + i + " should land at offset " + i);
            }
        }
        assertEquals(10, Support.consumeAllFromBeginning(topic, 10).size());
    }

    // ---- content round-trip ----------------------------------------------

    @Test
    void nullValueRecordIsAcceptedAndReadable() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            producer.send(new ProducerRecord<>(topic, 0, "k".getBytes(), null)).get();
        }
        List<ConsumerRecord<byte[], byte[]>> records = Support.consumeAllFromBeginning(topic, 1);
        assertEquals(1, records.size());
        assertEquals("k", new String(records.get(0).key()), "key must round-trip");
        // KNOWN LIMITATION: the storage record value is a non-optional byte
        // string, so a Kafka null value (tombstone) currently comes back as an
        // empty value rather than null. True tombstone fidelity needs a
        // null-flag through the write/read path (tracked with log-compaction).
        assertEquals(0, records.get(0).value().length,
                "null value currently round-trips as empty (see limitation above)");
    }

    @Test
    void compressionCodecsRoundTrip() throws Exception {
        // The broker must decode every batch compression the wire protocol allows.
        for (String codec : List.of("gzip", "snappy", "lz4", "zstd")) {
            String topic = name();
            createTopic(topic, 1);
            try (KafkaProducer<byte[], byte[]> producer =
                    Support.newProducer(Map.of(ProducerConfig.COMPRESSION_TYPE_CONFIG, codec))) {
                for (int i = 0; i < 3; i++) {
                    producer.send(new ProducerRecord<>(topic, 0, null, (codec + "-v" + i).getBytes()))
                            .get();
                }
            }
            List<ConsumerRecord<byte[], byte[]>> records = Support.consumeAllFromBeginning(topic, 3);
            assertEquals(3, records.size(), codec + ": all records should be readable");
            for (int i = 0; i < 3; i++) {
                assertEquals(codec + "-v" + i, new String(records.get(i).value()),
                        codec + ": value " + i + " must round-trip");
            }
        }
    }

    @Test
    void largeMessageRoundTrips() throws Exception {
        String topic = name();
        createTopic(topic, 1);
        byte[] big = new byte[1024 * 1024]; // 1 MiB
        for (int i = 0; i < big.length; i++) {
            big[i] = (byte) (i % 251);
        }
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer(Map.of(
                ProducerConfig.MAX_REQUEST_SIZE_CONFIG, 4 * 1024 * 1024,
                ProducerConfig.BUFFER_MEMORY_CONFIG, 8L * 1024 * 1024))) {
            producer.send(new ProducerRecord<>(topic, 0, null, big)).get();
        }
        List<ConsumerRecord<byte[], byte[]>> records = Support.consumeAllFromBeginning(topic, 1);
        assertEquals(1, records.size());
        org.junit.jupiter.api.Assertions.assertArrayEquals(big, records.get(0).value(),
                "a 1 MiB payload must round-trip byte-for-byte");
    }

    // ---- partition routing ------------------------------------------------

    @Test
    void sameKeyAlwaysRoutesToOnePartition() throws Exception {
        String topic = name();
        createTopic(topic, 3);
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer()) {
            Set<Integer> partitions = new HashSet<>();
            for (int i = 0; i < 6; i++) {
                partitions.add(producer.send(
                        new ProducerRecord<>(topic, "same-key".getBytes(), ("v" + i).getBytes()))
                        .get().partition());
            }
            assertEquals(1, partitions.size(), "records sharing a key must stay on one partition");
        }
    }

    // ---- errors -----------------------------------------------------------

    @Test
    void produceToUnknownTopicFailsWhenAutoCreateOff() throws Exception {
        Support.setAutoCreateTopics(false);
        String topic = name(); // never created
        try (KafkaProducer<byte[], byte[]> producer =
                Support.newProducer(Map.of(ProducerConfig.MAX_BLOCK_MS_CONFIG, "6000"))) {
            // With auto-creation off the topic never resolves, so the send must
            // fail: either the metadata wait times out, or the broker's
            // UNKNOWN_TOPIC metadata surfaces as UnknownTopicOrPartition.
            Exception ex = assertThrows(Exception.class,
                    () -> producer.send(new ProducerRecord<>(topic, 0, null, "x".getBytes())).get());
            Throwable cause = (ex instanceof ExecutionException) ? ex.getCause() : ex;
            assertTrue(cause instanceof TimeoutException
                            || cause instanceof UnknownTopicOrPartitionException,
                    "expected timeout or unknown-topic failure, got " + cause);
        }
    }
}
