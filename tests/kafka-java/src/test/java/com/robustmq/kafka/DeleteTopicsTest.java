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

import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.common.errors.UnknownTopicOrPartitionException;
import org.junit.jupiter.api.Test;

class DeleteTopicsTest {

    private static String name() {
        return "it-delete-" + UUID.randomUUID();
    }

    // Cache removal propagates via a notify after the delete is persisted, so the
    // topic may linger briefly in the listing; poll a few times.
    private static boolean waitUntilAbsent(Admin admin, String topic) throws Exception {
        for (int i = 0; i < 20; i++) {
            if (!admin.listTopics().names().get().contains(topic)) {
                return true;
            }
            Thread.sleep(250);
        }
        return false;
    }

    @Test
    void deleteExistingTopicRemovesIt() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 2, (short) 1))).all().get();
            assertTrue(admin.listTopics().names().get().contains(topic), "topic missing after create");

            admin.deleteTopics(List.of(topic)).all().get();

            assertTrue(waitUntilAbsent(admin, topic), "topic still present after delete");
        }
    }

    @Test
    void deleteUnknownTopicFails() throws Exception {
        String missing = name();
        try (Admin admin = Support.newAdmin()) {
            ExecutionException ex = assertThrows(
                    ExecutionException.class,
                    () -> admin.deleteTopics(List.of(missing)).all().get());
            assertInstanceOf(
                    UnknownTopicOrPartitionException.class,
                    ex.getCause(),
                    "expected UnknownTopicOrPartition, got " + ex.getCause());
        }
    }

    @Test
    void deleteMultipleTopics() throws Exception {
        String a = name();
        String b = name();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(
                    new NewTopic(a, 1, (short) 1),
                    new NewTopic(b, 1, (short) 1))).all().get();

            admin.deleteTopics(List.of(a, b)).all().get();

            Set<String> names = admin.listTopics().names().get();
            for (int i = 0; (names.contains(a) || names.contains(b)) && i < 20; i++) {
                Thread.sleep(250);
                names = admin.listTopics().names().get();
            }
            assertTrue(!names.contains(a) && !names.contains(b), "topics still present after delete");
        }
    }
}
