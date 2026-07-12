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

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Properties;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.AdminClientConfig;
import org.apache.kafka.clients.consumer.ConsumerConfig;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerConfig;
import org.apache.kafka.common.PartitionInfo;
import org.apache.kafka.common.TopicPartition;
import org.apache.kafka.common.serialization.ByteArrayDeserializer;
import org.apache.kafka.common.serialization.ByteArraySerializer;

/** Shared helpers for the Kafka integration tests. */
final class Support {
    private Support() {}

    static String bootstrapServers() {
        String v = System.getProperty("bootstrap.servers", System.getenv("KAFKA_BOOTSTRAP_SERVERS"));
        return v != null ? v : "localhost:9092";
    }

    static String adminUrl() {
        String v = System.getProperty("admin.url", System.getenv("ROBUSTMQ_ADMIN_ADDR"));
        return v != null ? v : "http://127.0.0.1:58080";
    }

    static Admin newAdmin() {
        return newAdmin(java.util.Map.of());
    }

    static Admin newAdmin(java.util.Map<String, Object> overrides) {
        Properties props = new Properties();
        props.put(AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers());
        props.put(AdminClientConfig.REQUEST_TIMEOUT_MS_CONFIG, 10_000);
        props.putAll(overrides);
        return Admin.create(props);
    }

    static KafkaProducer<byte[], byte[]> newProducer() {
        return newProducer(java.util.Map.of());
    }

    /** A byte[] producer with the safe defaults, plus any per-test overrides. */
    static KafkaProducer<byte[], byte[]> newProducer(java.util.Map<String, Object> overrides) {
        Properties props = new Properties();
        props.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers());
        props.put(ProducerConfig.ACKS_CONFIG, "all");
        // Default to non-idempotent (plain Produce) so these helpers exercise the
        // simple path; idempotence is supported (see IdempotentProduceTest) and
        // tests that want it override ENABLE_IDEMPOTENCE_CONFIG back to true.
        props.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, false);
        props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class.getName());
        props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class.getName());
        props.putAll(overrides);
        return new KafkaProducer<>(props);
    }

    /**
     * A plain byte[] consumer: reads from the earliest offset, no auto-commit.
     * `groupId` may be null for manual `assign()` usage that never talks to a
     * group coordinator; pass a real id for `subscribe()`-based group tests.
     */
    static KafkaConsumer<byte[], byte[]> newConsumer(String groupId) {
        return newConsumer(groupId, java.util.Map.of());
    }

    /** A byte[] consumer (earliest, no auto-commit) plus any per-test overrides. */
    static KafkaConsumer<byte[], byte[]> newConsumer(String groupId, java.util.Map<String, Object> overrides) {
        Properties props = new Properties();
        props.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers());
        if (groupId != null) {
            props.put(ConsumerConfig.GROUP_ID_CONFIG, groupId);
        }
        props.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        props.put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, "false");
        props.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, ByteArrayDeserializer.class.getName());
        props.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, ByteArrayDeserializer.class.getName());
        props.putAll(overrides);
        return new KafkaConsumer<>(props);
    }

    /** A consumer that opts into auto topic creation on metadata lookups. */
    static KafkaConsumer<byte[], byte[]> newAutoCreateConsumer() {
        Properties props = new Properties();
        props.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers());
        props.put(ConsumerConfig.GROUP_ID_CONFIG, "it-" + System.nanoTime());
        props.put(ConsumerConfig.ALLOW_AUTO_CREATE_TOPICS_CONFIG, "true");
        props.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, ByteArrayDeserializer.class.getName());
        props.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, ByteArrayDeserializer.class.getName());
        return new KafkaConsumer<>(props);
    }

    /**
     * Read every partition of {@code topic} from the beginning with a manual
     * {@code assign()} (no group), draining until {@code expected} records
     * arrive or the deadline passes. Used to verify produced content round-trips.
     */
    static List<ConsumerRecord<byte[], byte[]>> consumeAllFromBeginning(String topic, int expected) {
        try (KafkaConsumer<byte[], byte[]> consumer = newConsumer(null)) {
            List<TopicPartition> tps = new ArrayList<>();
            for (PartitionInfo pi : consumer.partitionsFor(topic)) {
                tps.add(new TopicPartition(topic, pi.partition()));
            }
            consumer.assign(tps);
            consumer.seekToBeginning(tps);
            List<ConsumerRecord<byte[], byte[]>> out = new ArrayList<>();
            long deadline = System.nanoTime() + Duration.ofSeconds(15).toNanos();
            while (out.size() < expected && System.nanoTime() < deadline) {
                for (ConsumerRecord<byte[], byte[]> r : consumer.poll(Duration.ofMillis(300))) {
                    out.add(r);
                }
            }
            return out;
        }
    }

    /** Toggle the cluster-level Kafka `auto_create_topics_enable` dynamic config via the admin HTTP API. */
    static void setAutoCreateTopics(boolean enabled) throws Exception {
        String body = "{\"config_type\":\"KafkaDynamic\",\"config\":\"{\\\"auto_create_topics_enable\\\":"
                + enabled + "}\"}";
        HttpClient client = HttpClient.newHttpClient();
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(adminUrl() + "/api/cluster/config/set"))
                .timeout(Duration.ofSeconds(10))
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body))
                .build();
        HttpResponse<String> resp = client.send(request, HttpResponse.BodyHandlers.ofString());
        if (resp.statusCode() != 200) {
            throw new IllegalStateException("set KafkaDynamic failed: HTTP " + resp.statusCode() + " " + resp.body());
        }
    }
}
