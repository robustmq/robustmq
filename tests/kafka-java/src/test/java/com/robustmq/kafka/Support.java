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
import java.util.Properties;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.AdminClientConfig;
import org.apache.kafka.clients.consumer.ConsumerConfig;
import org.apache.kafka.clients.consumer.KafkaConsumer;
import org.apache.kafka.common.serialization.ByteArrayDeserializer;

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
        Properties props = new Properties();
        props.put(AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers());
        props.put(AdminClientConfig.REQUEST_TIMEOUT_MS_CONFIG, 10_000);
        return Admin.create(props);
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
