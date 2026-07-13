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

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.AdminClientConfig;
import org.apache.kafka.common.TopicPartition;
import org.apache.kafka.common.errors.UnsupportedVersionException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

/**
 * Transactions are not implemented, so the transaction admin APIs and
 * DescribeProducers are deliberately not advertised. They must therefore fail
 * fast with UnsupportedVersion rather than hang (the old behaviour was a handler
 * returning no response, leaving the client waiting until timeout).
 */
class TransactionUnsupportedTest {

    private static Admin fastAdmin() {
        return Support.newAdmin(Map.of(
                AdminClientConfig.REQUEST_TIMEOUT_MS_CONFIG, 4000,
                AdminClientConfig.DEFAULT_API_TIMEOUT_MS_CONFIG, 8000));
    }

    private static void assertUnsupported(Executable call) {
        ExecutionException ex = assertThrows(ExecutionException.class, call);
        assertTrue(ex.getCause() instanceof UnsupportedVersionException,
                "unadvertised API should fail fast with UnsupportedVersion, got " + ex.getCause());
    }

    @Test
    void listTransactionsIsUnsupported() throws Exception {
        try (Admin admin = fastAdmin()) {
            assertUnsupported(() -> admin.listTransactions().all().get());
        }
    }

    @Test
    void describeTransactionsIsUnsupported() throws Exception {
        try (Admin admin = fastAdmin()) {
            assertUnsupported(() -> admin.describeTransactions(List.of("txn-" + UUID.randomUUID()))
                    .all().get());
        }
    }

    @Test
    void describeProducersIsUnsupported() throws Exception {
        String topic = "it-dp-" + UUID.randomUUID();
        try (Admin admin = fastAdmin()) {
            admin.createTopics(List.of(new org.apache.kafka.clients.admin.NewTopic(topic, 1, (short) 1)))
                    .all().get();
            assertUnsupported(() -> admin.describeProducers(List.of(new TopicPartition(topic, 0)))
                    .all().get());
        }
    }
}
