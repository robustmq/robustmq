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

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.Future;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.consumer.ConsumerRecord;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerConfig;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.clients.producer.RecordMetadata;
import org.junit.jupiter.api.Test;

/**
 * Phase 5 (idempotence only, no transactions): the default idempotent producer
 * (enable.idempotence=true) must obtain a producer id via InitProducerId(22) and
 * produce successfully. Transactions remain unimplemented.
 */
class IdempotentProduceTest {

    private static String name() {
        return "it-idem-" + UUID.randomUUID();
    }

    private static void createTopic(String topic, int partitions) throws Exception {
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
        }
    }

    private static KafkaProducer<byte[], byte[]> idempotentProducer() {
        return Support.newProducer(Map.of(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true));
    }

    @Test
    void idempotentProducerProducesAndReadsBack() throws Exception {
        String topic = name();
        createTopic(topic, 1);

        try (KafkaProducer<byte[], byte[]> producer = idempotentProducer()) {
            for (int i = 0; i < 5; i++) {
                RecordMetadata md =
                        producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes()))
                                .get();
                assertEquals((long) i, md.offset(), "idempotent produce should assign offset " + i);
            }
        }

        List<ConsumerRecord<byte[], byte[]>> records = Support.consumeAllFromBeginning(topic, 5);
        assertEquals(5, records.size(), "every idempotent record should be readable");
        for (int i = 0; i < 5; i++) {
            assertEquals("v" + i, new String(records.get(i).value()));
        }
    }

    @Test
    void idempotentBatchesGetContiguousOffsets() throws Exception {
        String topic = name();
        createTopic(topic, 1);

        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer(Map.of(
                ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true,
                ProducerConfig.LINGER_MS_CONFIG, "50",
                ProducerConfig.BATCH_SIZE_CONFIG, "1048576"))) {
            List<Future<RecordMetadata>> futures = new ArrayList<>();
            for (int i = 0; i < 10; i++) {
                futures.add(producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes())));
            }
            producer.flush();
            for (int i = 0; i < 10; i++) {
                assertEquals((long) i, futures.get(i).get().offset(),
                        "idempotent batched record " + i + " should land at offset " + i);
            }
        }
        assertEquals(10, Support.consumeAllFromBeginning(topic, 10).size());
    }
}
