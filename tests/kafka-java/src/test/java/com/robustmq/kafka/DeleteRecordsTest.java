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

import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.RecordsToDelete;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.TopicPartition;
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
}
